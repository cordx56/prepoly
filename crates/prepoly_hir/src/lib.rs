//! High-level IR for Prepoly: the type representation and the collected,
//! id-assigned program model the type checker annotates and the code generator
//! consumes.

pub mod hir;
pub mod lower;
pub mod typed;
pub mod types;

pub use hir::{
    CallableSignature, FieldInfo, FunInfo, LoadedModule, MethodInfo, ModuleInit, ParamInfo,
    Program, QualifiedName, RESULT_TYPE_ID, TypeInfo, TypeKind, VariantInfo, qualify,
    resolve_qualified,
};
pub use lower::{LowerError, lower};
pub use typed::{Constness, Ownership, RegionId, TypedExpr, TypedExprKind, TypedProgram};
pub use types::{
    FloatKind, IntKind, NominalInfo, NominalKind, NominalType, Substitution, Type, resolve,
};

/// Re-exported so back ends can name source spans (e.g. typed-literal codegen
/// keyed by span) without depending on the lexer crate directly.
pub use prepoly_lexer::Span;
