//! Value-flow unification leniency, shared by every unification-based pass
//! (the HM checker and the JIT-time MIR checker) so a flow the front end
//! accepted is never re-rejected later in the pipeline.

use brass_hir::{Type, index_element};
use brass_solver::solver::Solver;

/// A nullable's element type (one level), else the type unchanged. Value flow
/// treats `T` and `T?` as compatible: a `T` flows into a `T?` by promotion and
/// a guarded `T?` into a `T` by narrowing. Deeper null-safety is the flow
/// checker's concern, not unification's.
pub fn strip_nullable(ty: Type) -> Type {
    match ty {
        Type::Nullable(inner) => *inner,
        other => other,
    }
}

/// Unify with Brass's value-flow leniency, returning whether it succeeded.
/// A top-level nullable is stripped from each side (see [`strip_nullable`]),
/// and array-like types reconcile by element type -- a `[1,2,3]` literal,
/// inferred as a slice, matches an `int32[3]` annotation, seen through
/// `ref`/`mut`/`const` wrappers the same way indexing is.
///
/// Succeeding commits the unification's bindings; failing commits nothing (the
/// attempt is rolled back), so a caller can try further fallbacks -- numeric
/// conversion, structural subtyping -- against unpolluted state.
pub fn flow_unify(solver: &mut Solver, a: &Type, b: &Type) -> bool {
    let a = strip_nullable(solver.resolve(a));
    let b = strip_nullable(solver.resolve(b));
    if let (Some(x), Some(y)) = (index_element(&a), index_element(&b)) {
        return flow_unify(solver, &x, &y);
    }
    let snapshot = solver.snapshot();
    if solver.unify(&a, &b).is_ok() {
        return true;
    }
    solver.rollback(snapshot);
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use brass_hir::IntKind;
    use brass_solver::solver::InferenceVarKind;

    /// A nullable reconciles with its element, and array-likes reconcile by
    /// element type even under a reference wrapper (the indexing rule).
    #[test]
    fn nullable_and_wrapped_arrays_reconcile() {
        let mut solver = Solver::new();
        let int = Type::Int(IntKind::I32);
        assert!(flow_unify(
            &mut solver,
            &Type::Nullable(Box::new(int.clone())),
            &int
        ));
        let arr = Type::Slice(Box::new(int.clone()));
        let fixed = Type::Array(Box::new(int.clone()), 3);
        assert!(flow_unify(
            &mut solver,
            &Type::Ref(Box::new(arr)),
            &Type::Ref(Box::new(fixed))
        ));
    }

    /// A failed flow unification must commit nothing: the variable bound while
    /// the attempt was in flight is unbound again afterwards, so a caller's
    /// fallback (numeric conversion, structural subtyping) sees clean state.
    #[test]
    fn failed_unification_commits_no_bindings() {
        let mut solver = Solver::new();
        let v = solver.fresh(InferenceVarKind::Source);
        // [v, string] vs [int32, bool]: v binds to int32 first, then the
        // string/bool element fails the whole unification.
        let a = Type::Tuple(vec![v.clone(), Type::Str]);
        let b = Type::Tuple(vec![Type::Int(IntKind::I32), Type::Bool]);
        assert!(!flow_unify(&mut solver, &a, &b));
        assert!(
            solver.resolve(&v).is_unknown(),
            "failed flow_unify leaked a partial binding: {:?}",
            solver.resolve(&v)
        );
    }
}
