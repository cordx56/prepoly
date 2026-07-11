//! Stands in for a rebuilt `prepoly_plugin_testlib`: `add` behaves the same,
//! and `extra` is the function a stale manifest would not know about.

use prepoly_plugin::{PrepolyLib, Registry, decl, export, prepoly_lib};

export! {
    /// Adds two integers.
    fn add(a: i64, b: i64) -> i64 { a.wrapping_add(b) }

    /// Only this build exposes it.
    fn extra() -> i64 { 7 }
}

struct AltLib;

impl PrepolyLib for AltLib {
    fn entry(reg: &mut Registry) {
        reg.export(decl!(add));
        reg.export(decl!(extra));
    }
}

prepoly_lib!(AltLib);
