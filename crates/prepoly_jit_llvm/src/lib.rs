//! The LLVM JIT back end for Prepoly.
//!
//! This crate generates typed, fully unboxed LLVM IR (the [`codegen`] module
//! implementing `prepoly_engine::Codegen` over monomorphized MIR) and runs it:
//! the [`jit`] module owns OrcJIT execution. The entry point [`jit::run`] is
//! re-exported at the crate root.
//!
pub mod closure;
pub mod codegen;
pub mod dispatch;
pub mod jit;
pub mod layout;
pub mod monomorph;
pub mod ownership;

pub use codegen::LlvmCodegen;
pub use dispatch::{RuntimeDispatcher, pp_resolve, with_dispatcher};
pub use jit::run;
pub use monomorph::{mangle_closure, mangle_fn, mangle_init, mangle_method};
