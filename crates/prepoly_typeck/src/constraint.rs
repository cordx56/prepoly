//! Deferred structural constraints on inference variables.
//!
//! A closure is type-checked once, at its definition, with its unannotated
//! parameters bound to fresh inference variables. Its body may use a parameter
//! in a way that constrains it (arithmetic, a method/field access, indexing)
//! even though the concrete argument type is only known later, at a call site.
//!
//! Each such use records a `ShapeConstraint` keyed by the parameter's inference
//! variable id. When the closure value is applied (`apply_callable`), the
//! argument is unified into the parameter and every recorded constraint is
//! verified against the now-concrete type. This makes a closure such as
//! `(x) -> x + 1` reject a `string` argument, matching the soundness a named
//! function already gets from per-call body re-checking.

use prepoly_hir::Type;

/// A structural requirement on an inference variable, discovered while checking
/// a body that used the variable before its concrete type was known. Verified
/// when the variable is solved at a call site.
#[derive(Clone, Debug)]
pub enum ShapeConstraint {
    /// The variable must equal a concrete type. Recorded from a same-typed
    /// binary operator whose other operand is concrete (`x + 1` => `x` is
    /// `int32`), since Prepoly performs no implicit numeric conversion.
    Equals(Type),
    /// The variable must be a value exposing the named method (`x.speak()`).
    HasMethod(String),
    /// The variable must be a record exposing the named field (`x.name`).
    HasField(String),
    /// The variable must be indexable: an array, slice, or string (`x[0]`).
    Indexable,
}
