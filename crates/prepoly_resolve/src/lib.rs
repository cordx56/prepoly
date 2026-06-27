//! Name resolution for Prepoly: module imports, visibility, and a reusable
//! lexical scope.

pub mod module;
pub mod scope;
pub mod visibility;

pub use module::{ResolveError, check_imports};
pub use scope::Scope;
pub use visibility::{is_private_module, is_public};
