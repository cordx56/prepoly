//! Match exhaustiveness (DESIGN.md 4.6): a match whose arms reference variants
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
    fn owning_sum(&self, vname: &str) -> Option<String> {
        for (tn, info) in &self.program.types {
            if let TypeKind::Sum { variants } = &info.kind
                && variants.iter().any(|v| v.name == vname)
            {
                return Some(tn.clone());
            }
        }
        None
    }

    fn check_match(&mut self, arms: &[MatchArm], span: Span) {
        let mut owner = None;
        for a in arms {
            let name = match &a.pattern {
                Pattern::Record(n, _, _) | Pattern::Binding(n, _) => Some(n.as_str()),
                _ => None,
            };
            if let Some(n) = name
                && let Some(t) = self.owning_sum(n)
            {
                owner = Some(t);
                break;
            }
        }
        let Some(owner) = owner else { return };
        let TypeKind::Sum { variants } = &self.program.types[&owner].kind else {
            return;
        };
        let all: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
        let mut covered: HashSet<String> = HashSet::new();
        let mut catch_all = false;
        for a in arms {
            match &a.pattern {
                Pattern::Wildcard(_) => catch_all = true,
                Pattern::Record(n, _, _) => {
                    covered.insert(n.clone());
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
