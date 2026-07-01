//! The Prepoly REPL execution back end.
//!
//! This crate runs a checked program without LLVM: it reuses
//! `prepoly_engine`'s monomorphization to produce a [`prepoly_engine::MonoProgram`]
//! of concretely-typed instances, then *interprets* that MIR by walking the
//! control-flow graph (see [`interp`]). It is the alternative to the typed LLVM JIT
//! (`prepoly_jit_llvm`): it depends only on the backend-agnostic engine, so it
//! builds and runs on any host. The interpreter covers the typed sequential subset
//! (scalars, strings, arrays, records, sums, closures, nullable/`Result`, control
//! flow, recursion); runtime features outside that subset (concurrency, file I/O,
//! deferred type dispatch) report a clear error rather than executing.

mod format;
mod interp;
mod value;

use std::io::Write;

use prepoly_hir::Program;

pub use interp::Interp;
pub use value::Value;

/// Compile a checked program to monomorphized MIR and interpret it, writing its
/// `print`/`println` output to `out`. Mirrors `prepoly_jit_llvm::run`'s contract:
/// a program whose `main` reaches a construct outside the typed subset is rejected
/// rather than partially executed.
pub fn run(
    program: &Program,
    expr_types: &std::collections::HashMap<prepoly_hir::Span, prepoly_hir::Type>,
    out: &mut dyn Write,
) -> Result<(), String> {
    let mir = prepoly_mir::lower_program_with_types(program, expr_types);
    let mono = prepoly_engine::monomorphize(&mir, program)
        .map_err(|e| format!("typed lowering failed: {e}"))?;
    if program.functions.contains_key("main") && mono.lookup("main").is_none() {
        return Err(
            "program uses constructs outside the REPL runtime's supported subset".to_string(),
        );
    }
    let mut interp = Interp::new(&mono, program, out);
    interp.run()
}
