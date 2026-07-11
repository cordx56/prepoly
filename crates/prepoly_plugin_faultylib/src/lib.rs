//! A plugin whose `entry` panics. The host loads plugins while resolving an
//! `import`, inside the compiler and inside the language server, so this
//! failure must arrive as a load error rather than unwinding across the C ABI.

use prepoly_plugin::{PrepolyLib, Registry, prepoly_lib};

struct FaultyLib;

impl PrepolyLib for FaultyLib {
    fn entry(_reg: &mut Registry) {
        panic!("this plugin cannot register itself");
    }
}

prepoly_lib!(FaultyLib);
