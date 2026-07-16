//! A plugin whose `entry` panics. The host loads plugins while resolving an
//! `import`, inside the compiler and inside the language server, so this
//! failure must arrive as a load error rather than unwinding across the C ABI.

use brass_plugin::{BrassLib, Registry, brass_lib};

struct FaultyLib;

impl BrassLib for FaultyLib {
    fn entry(_reg: &mut Registry) {
        panic!("this plugin cannot register itself");
    }
}

brass_lib!(FaultyLib);
