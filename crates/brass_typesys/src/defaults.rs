//! Default-skeleton constructibility. An uninitialized `let x: T` that is
//! initialized field by field needs a default-valued skeleton allocation for
//! the field stores to have a target; MIR lowering synthesizes it (see
//! `brass_mir::lower`'s `default_operand`, which mirrors this decision).
//! This predicate is the single authority for WHICH types have such a
//! skeleton: the checker consults it to reject field-wise initialization of a
//! type the lowering could not materialize.

use brass_hir::{Program, Type, TypeKind};

/// Whether `ty` has a constructible default skeleton: zero for numbers,
/// `false`, `""`, `null` for nullables, `[]` for slices, element-wise defaults
/// for fixed arrays and tuples, and field-wise defaults for records. Sum
/// types and function values have no default; a record is only constructible
/// when every field (transitively) is, and an infinitely recursive record
/// (`type A = { b: B }`, `type B = { a: A }`) is not.
pub fn default_constructible(program: &Program, ty: &Type) -> bool {
    constructible(program, ty, &mut Vec::new())
}

fn constructible(program: &Program, ty: &Type, visiting: &mut Vec<i32>) -> bool {
    match ty {
        Type::Int(_) | Type::Float(_) | Type::Bool | Type::Str | Type::Nullable(_) => true,
        Type::Slice(_) => true,
        Type::Array(e, _) => constructible(program, e, visiting),
        Type::Tuple(ts) => ts.iter().all(|t| constructible(program, t, visiting)),
        Type::Record(n) => {
            if visiting.contains(&n.id) {
                return false;
            }
            visiting.push(n.id);
            let ok = program
                .types
                .values()
                .find(|i| i.id == n.id)
                .is_some_and(|info| match &info.kind {
                    TypeKind::Record { fields, .. } => fields.iter().all(|f| {
                        f.resolved_ty
                            .as_ref()
                            .is_some_and(|ft| constructible(program, ft, visiting))
                    }),
                    _ => false,
                });
            visiting.pop();
            ok
        }
        _ => false,
    }
}
