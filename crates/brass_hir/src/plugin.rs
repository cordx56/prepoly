//! The native-plugin value-type code, decoded into Brass types.
//!
//! A plugin's boundary types are written as a compact code -- one letter per
//! leaf type, with an `a` prefix per array level -- so the loader can splice
//! them into the `_plugin_[f]call_<code>` builtin name and into the signature
//! string literal that builtin carries (see `brass_resolve::plugin`). Both
//! spellings are decoded here, once, so the checker, the back ends, and the
//! interpreter cannot drift from `brass_plugin::ValueType`, which owns the
//! encoding side. This module lives in `brass_hir` because the checker must
//! not depend on the plugin host.

use crate::types::{FloatKind, IntKind, Type};

/// The parameter types, return type, and fallibility encoded by a signature
/// string (`"ii:i!"`), or `None` when it is malformed. Type codes are
/// self-delimiting, so the parameter list needs no separators.
pub fn plugin_sig_types(sig: &str) -> Option<(Vec<Type>, Type, bool)> {
    let (params, ret) = sig.split_once(':')?;
    let mut chars = params.chars();
    let mut param_types = Vec::new();
    while chars.clone().next().is_some() {
        param_types.push(plugin_type_code(&mut chars)?);
    }
    let fallible = ret.ends_with('!');
    let ret = ret.strip_suffix('!').unwrap_or(ret);
    let mut chars = ret.chars();
    let ret = plugin_type_code(&mut chars)?;
    if chars.next().is_some() {
        return None;
    }
    Some((param_types, ret, fallible))
}

/// The return type of a `_plugin_call_<code>` / `_plugin_fcall_<code>`
/// builtin (the native-plugin dispatch calls the loader synthesizes), or
/// `None` when `name` is not one. The `_fcall_` family is fallible and
/// returns `Result<T, string>`.
pub fn plugin_builtin_return(name: &str) -> Option<Type> {
    let (code, fallible) = plugin_builtin_code(name)?;
    let mut chars = code.chars();
    let payload = plugin_type_code(&mut chars)?;
    if chars.next().is_some() {
        return None;
    }
    Some(if fallible {
        Type::result(payload, Type::Str)
    } else {
        payload
    })
}

/// The return-type code and fallibility spliced into a plugin builtin's name.
pub fn plugin_builtin_code(name: &str) -> Option<(&str, bool)> {
    if let Some(rest) = name.strip_prefix("_plugin_call_") {
        Some((rest, false))
    } else if let Some(rest) = name.strip_prefix("_plugin_fcall_") {
        Some((rest, true))
    } else {
        None
    }
}

/// Decode one plugin value-type code from the front of `chars`.
pub fn plugin_type_code(chars: &mut std::str::Chars<'_>) -> Option<Type> {
    Some(match chars.next()? {
        'v' => Type::Void,
        'b' => Type::Bool,
        'i' => Type::Int(IntKind::I64),
        'f' => Type::Float(FloatKind::F64),
        's' => Type::Str,
        // `uint8[]` is its own code: the boundary's only integer is `int64`,
        // so a byte buffer is not `Array(int)`.
        'y' => Type::Slice(Box::new(Type::Int(IntKind::U8))),
        'a' => Type::Slice(Box::new(plugin_type_code(chars)?)),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes() -> Type {
        Type::Slice(Box::new(Type::Int(IntKind::U8)))
    }

    /// The plugin dispatch builtins carry their return type in the name: one
    /// letter per leaf, an `a` per array level, and the `_fcall_` family wraps
    /// in `Result`. A name that is not one of them, or carries a malformed
    /// code, decodes to nothing.
    #[test]
    fn plugin_builtin_names_decode_their_return_type() {
        assert_eq!(
            plugin_builtin_return("_plugin_call_i"),
            Some(Type::Int(IntKind::I64))
        );
        assert_eq!(plugin_builtin_return("_plugin_call_y"), Some(bytes()));
        assert_eq!(
            plugin_builtin_return("_plugin_call_aas"),
            Some(Type::Slice(Box::new(Type::Slice(Box::new(Type::Str)))))
        );
        assert_eq!(
            plugin_builtin_return("_plugin_fcall_s"),
            Some(Type::result(Type::Str, Type::Str))
        );
        assert_eq!(plugin_builtin_return("_plugin_call_a"), None);
        assert_eq!(plugin_builtin_return("_plugin_call_si"), None);
        assert_eq!(plugin_builtin_return("_tls_connect"), None);
    }

    /// A signature decodes its unseparated parameter codes, the return type,
    /// and the `!` fallibility marker; `y` is `uint8[]`, distinct from `ai`.
    #[test]
    fn signature_decoding() {
        assert_eq!(
            plugin_sig_types("isy:b!"),
            Some((
                vec![Type::Int(IntKind::I64), Type::Str, bytes()],
                Type::Bool,
                true
            ))
        );
        assert_eq!(plugin_sig_types(":v"), Some((vec![], Type::Void, false)));
        assert_eq!(plugin_sig_types("i"), None);
        assert_eq!(plugin_sig_types("q:i"), None);
        assert_eq!(plugin_sig_types("i:ii"), None);
    }
}
