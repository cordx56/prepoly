//! Selects the execution back end at build time.
//!
//! The JIT back end (LLVM via `prepoly_jit_llvm`) is available only when the `jit`
//! feature is on AND the target is not wasm -- LLVM cannot link for wasm. The `jit`
//! feature is on by default, so this script turns it off automatically for a wasm
//! target by *not* emitting the `jit_backend` cfg there. The source then selects
//! the back end on that single cfg, and the LLVM dependencies (declared only for
//! non-wasm targets in Cargo.toml) are never pulled into a wasm build.

fn main() {
    println!("cargo::rustc-check-cfg=cfg(jit_backend)");

    let jit_feature = std::env::var_os("CARGO_FEATURE_JIT").is_some();
    let is_wasm = std::env::var("CARGO_CFG_TARGET_FAMILY")
        .map(|families| families.split(',').any(|f| f == "wasm"))
        .unwrap_or(false);

    if jit_feature && !is_wasm {
        println!("cargo::rustc-cfg=jit_backend");
    }
}
