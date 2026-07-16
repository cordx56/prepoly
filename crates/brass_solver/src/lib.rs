//! The Hindley-Milner constraint solver: a persistent union-find substitution
//! over `brass_hir::Type` with occurs check, snapshot/rollback, and type
//! schemes (generalize/instantiate). It is shared by the AST-level checker
//! (`brass_typeck`) and the JIT-time MIR inference (`brass_engine`), so both
//! resolve type variables with the same unification semantics.

pub mod solver;
pub mod unify;
