//! Match exhaustiveness: a match whose arms reference variants
//! of a sum type must cover every variant unless a catch-all (wildcard or
//! whole-value binding) is present.

use std::collections::HashSet;

use prepoly_hir::{Program, TypeKind};
use prepoly_lexer::Span;
use prepoly_parser::ast::*;

use crate::TypeError;
use crate::walk::{ExprVisitor, walk_program_exprs};

pub fn check(program: &Program) -> Vec<TypeError> {
    let mut v = ExhaustiveVisitor {
        program,
        errors: Vec::new(),
    };
    walk_program_exprs(program, &mut v);
    v.errors
}

/// Whether a bare name in a pattern names some sum's variant (a refutable
/// unit-variant test) rather than a plain binding.
fn is_variant_name(program: &Program, name: &str) -> bool {
    !program.sums_containing_variant(name).is_empty()
}

/// Whether a record field sub-pattern always matches. Shorthand `{ name }`
/// (no sub-pattern) binds the field and is irrefutable.
fn field_irrefutable(program: &Program, f: &FieldPat) -> bool {
    match &f.pat {
        None => true,
        Some(p) => pattern_irrefutable(program, p),
    }
}

/// Whether a pattern matches every value of its type, so an arm built from it
/// exhausts what it is matched against. A literal, a fixed-length array
/// destructure (refutable on length), a nested variant test, or any compound
/// containing one is refutable. Shared with the control-flow pass, which only
/// treats a match as diverging when an arm is guaranteed to run.
pub(crate) fn pattern_irrefutable(program: &Program, p: &Pattern) -> bool {
    match p {
        Pattern::Wildcard(_) => true,
        // A bare name is an irrefutable binding unless it names a variant, in
        // which case it is a refutable unit-variant test on the field.
        Pattern::Binding(n, _) => !is_variant_name(program, n),
        Pattern::Record(_, fields, _) => fields.iter().all(|f| field_irrefutable(program, f)),
        Pattern::Literal(_, _) | Pattern::Array(_, _) => false,
    }
}

/// Whether a top-level match arm pattern references a sum variant, meaning the
/// match is covered by this pass's sum-coverage check. Shared with the
/// control-flow pass: a match with neither a catch-all nor variant arms (e.g.
/// all integer literals) is never verified exhaustive, so control can fall
/// through every arm.
pub(crate) fn pattern_names_sum_variant(program: &Program, p: &Pattern) -> bool {
    match p {
        Pattern::Record(n, _, _) | Pattern::Binding(n, _) => is_variant_name(program, n),
        _ => false,
    }
}

struct ExhaustiveVisitor<'a> {
    program: &'a Program,
    errors: Vec<TypeError>,
}

impl ExprVisitor for ExhaustiveVisitor<'_> {
    fn visit(&mut self, e: &Expr) {
        if let Expr::Match(_, arms, span) = e {
            self.check_match(arms, *span);
        }
    }
}

impl ExhaustiveVisitor<'_> {
    /// The sum type the arms of a match are over. The scrutinee's type is not
    /// known in this AST pass, so the owner is chosen from the variant names
    /// the arms use: the sum containing *all* of them. When several sums
    /// qualify (shared variant names), the most specific one (fewest variants,
    /// then name) is picked -- a deterministic choice, so the type table's hash
    /// order never decides accept/reject; an ambiguous match is then judged
    /// against the sum most likely to make it exhaustive (conservative accept).
    fn owning_sum(&self, arms: &[MatchArm]) -> Option<String> {
        let variant_names: Vec<&str> = arms
            .iter()
            .filter_map(|a| match &a.pattern {
                Pattern::Record(n, _, _) | Pattern::Binding(n, _) => {
                    is_variant_name(self.program, n).then_some(n.as_str())
                }
                _ => None,
            })
            .collect();
        let first = variant_names.first()?;
        self.program
            .sums_containing_variant(first)
            .into_iter()
            .filter(|info| variant_names.iter().all(|n| info.variant(n).is_some()))
            .min_by_key(|info| match &info.kind {
                TypeKind::Sum { variants } => (variants.len(), info.name.clone()),
                TypeKind::Record { .. } => (usize::MAX, info.name.clone()),
            })
            .map(|info| info.symbol.clone())
    }

    fn check_match(&mut self, arms: &[MatchArm], span: Span) {
        let Some(owner) = self.owning_sum(arms) else {
            return;
        };
        let TypeKind::Sum { variants } = &self.program.types[&owner].kind else {
            return;
        };
        let all: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
        let mut covered: HashSet<String> = HashSet::new();
        let mut catch_all = false;
        for a in arms {
            match &a.pattern {
                Pattern::Wildcard(_) => catch_all = true,
                Pattern::Record(n, fields, _) => {
                    // A variant pattern covers the variant only when every field
                    // sub-pattern is irrefutable; a refutable one (e.g. a literal
                    // `Both { left: 1, .. }`) matches only some of the variant's
                    // values, so the rest must still be handled by another arm or a
                    // catch-all -- otherwise an unmatched value reaches `unreachable`.
                    if fields.iter().all(|f| field_irrefutable(self.program, f)) {
                        covered.insert(n.clone());
                    }
                }
                Pattern::Binding(n, _) => {
                    if all.contains(n) {
                        covered.insert(n.clone());
                    } else {
                        catch_all = true;
                    }
                }
                _ => {}
            }
        }
        if !catch_all {
            let missing: Vec<&str> = all
                .iter()
                .filter(|v| !covered.contains(*v))
                .map(|s| s.as_str())
                .collect();
            if !missing.is_empty() {
                tracing::debug!(sum = %owner, missing = ?missing, "non-exhaustive match");
                self.errors.push(TypeError {
                    message: format!(
                        "non-exhaustive match on `{owner}`: missing variant(s) {}",
                        missing.join(", ")
                    ),
                    span,
                });
            }
        }
    }
}
