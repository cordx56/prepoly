//! Resolution of the reflective conversion `infer.from(x)`: which concrete
//! conversion (if any) the call reaches for a given (source, target) pair.
//!
//! The decision is deliberately NARROWER than the set of `T.from` forms the
//! language exposes directly: the reflective path follows the from/parse
//! partition (from = value conversion, parse = interpretation from text,
//! interpolation = formatting to text), so a JSON string can never silently
//! become a number and a JSON number can never silently become a string.
//! `Absent` means no conversion exists for the pair -- at a decoder's `return`
//! position the specializer turns it into a runtime decode error.

use prepoly_hir::{Program, Type, TypeKind};

/// How `infer.from(x)` resolves for source type `from` and key (target) `to`.
#[derive(Clone, Debug, PartialEq)]
pub enum InferFrom {
    /// The source already is the target; the call is the argument itself.
    Identity,
    /// Resolves to the static call `<qualifier>.from(x)`.
    Static { qualifier: String },
    /// No conversion exists for this pair.
    Absent(String),
}

/// Resolve `infer.from(x)` for a source value of type `from` at key `to`.
pub fn infer_from(program: &Program, from: &Type, to: &Type) -> InferFrom {
    let from = prepoly_hir::peel_modes(from);
    match to {
        Type::Nullable(inner) => match infer_from(program, from, inner) {
            InferFrom::Identity => InferFrom::Identity,
            InferFrom::Static { qualifier } => InferFrom::Static { qualifier },
            absent => absent,
        },
        _ if from == to => InferFrom::Identity,
        Type::Int(k) => match from {
            Type::Int(_) | Type::Float(_) => InferFrom::Static {
                qualifier: k.name().to_string(),
            },
            _ => absent(from, to),
        },
        Type::Float(k) => match from {
            Type::Int(_) | Type::Float(_) => InferFrom::Static {
                qualifier: k.name().to_string(),
            },
            _ => absent(from, to),
        },
        // Text and booleans accept only their own type (handled by the Identity
        // case above); number->string is interpolation, not `from`.
        Type::Str | Type::Bool => absent(from, to),
        // A nominal target resolves through a user-defined static `from` method
        // when one exists; the builtin structural record-from is deliberately
        // NOT reachable reflectively (it yields null on a miss rather than a
        // decode error).
        Type::Record(n) | Type::Sum(n) => {
            let has_from = program
                .types
                .values()
                .find(|i| i.id == n.id)
                .map(|info| match &info.kind {
                    TypeKind::Record { methods, .. } => methods.contains_key("from"),
                    TypeKind::Sum { variants } => {
                        variants.iter().any(|v| v.methods.contains_key("from"))
                    }
                })
                .unwrap_or(false);
            if has_from {
                InferFrom::Static {
                    qualifier: to.type_name(),
                }
            } else {
                absent(from, to)
            }
        }
        _ => absent(from, to),
    }
}

fn absent(from: &Type, to: &Type) -> InferFrom {
    InferFrom::Absent(format!(
        "no conversion from `{}` to `{}`",
        from.display(),
        to.display()
    ))
}
