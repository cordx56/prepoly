//! Instance symbol naming: every monomorphized function, method,
//! constructor, and closure instance gets a collision-free symbol
//! derived from its base name and concrete type arguments.

use super::*;

/// Marks a compiler-synthesized instance symbol (a module init, method, static,
/// or closure) so it occupies a namespace disjoint from user function symbols.
/// `$` cannot appear in a source identifier, so a user `fun init0` (symbol
/// `init0`) never collides with the first module init (`$init0`) in the instance
/// map, and the back end maps the two to distinct LLVM names. Must match the
/// prefix `prepoly_jit_llvm`'s `mangle_fn` recognizes.
pub const SYNTH_SIGIL: char = '$';

/// The canonical instance symbol for `base` specialized to `type_args`. Distinct
/// type tuples yield distinct strings, so instances never collide: the argument
/// section is a `$$`-prefixed join of collision-resistant [`type_key`]s (no base
/// symbol contains `$$`, and no key contains `$`, so the base/argument split is
/// unambiguous -- a display-based join would let `(A_B, C)` and `(A, B_C)` share
/// a symbol and silently reuse a body typed for different argument types).
pub fn instance_symbol(base: &str, type_args: &[Type]) -> String {
    if type_args.is_empty() {
        base.to_string()
    } else {
        let args = type_args.iter().map(type_key).collect::<Vec<_>>().join("_");
        format!("{base}$${args}")
    }
}

/// A collision-resistant key for one concrete type inside an instance symbol.
/// Unlike `Type::display`, every token is self-delimiting (no `_` inside a
/// token), constructors carry their arity, nominal types are keyed by id (two
/// same-named types from different modules stay distinct), and substitution
/// field names are length-prefixed. Joining keys with `_` is therefore a
/// uniquely decodable code: equal symbols imply equal type tuples.
fn type_key(ty: &Type) -> String {
    match ty {
        Type::Bool => "bool".into(),
        Type::Int(k) => k.name().into(),
        Type::Float(k) => k.name().into(),
        Type::Str => "str".into(),
        Type::Void => "void".into(),
        Type::Never => "never".into(),
        Type::Record(n) => nominal_key("rec", n),
        Type::Sum(n) => nominal_key("sum", n),
        Type::Array(e, len) => format!("arr{len}_{}", type_key(e)),
        Type::Slice(e) => format!("slice_{}", type_key(e)),
        Type::Tuple(es) => {
            let mut out = format!("tup{}", es.len());
            for e in es {
                out.push('_');
                out.push_str(&type_key(e));
            }
            out
        }
        Type::Fun(ps, ret) => {
            let mut out = format!("fn{}", ps.len());
            for p in ps {
                out.push('_');
                out.push_str(&type_key(p));
            }
            out.push('_');
            out.push_str(&type_key(ret));
            out
        }
        Type::Nullable(inner) => format!("opt_{}", type_key(inner)),
        // Passing modes do not change the value's concrete layout.
        Type::ConstOf(inner) | Type::Mut(inner) | Type::Ref(inner) => type_key(inner),
        Type::Unknown(id) => format!("unk{id}"),
        Type::SelfType => "selfty".into(),
    }
}

/// Key of a record/sum: nominal id plus every substitution entry, so two
/// instantiations of one generic container (`HashMap<string,int32>` vs
/// `HashMap<int32,int32>`) -- and two structural records with different fields --
/// get distinct instance symbols. Field names are length-prefixed because a
/// source identifier may itself contain `_`.
fn nominal_key(tag: &str, n: &prepoly_hir::NominalType) -> String {
    let entries: Vec<(&str, &Type)> = n.substitution.iter().collect();
    let mut out = format!("{tag}{}x{}", n.id, entries.len());
    // A negative id is a shared internal id (e.g. every structural record uses
    // `STRUCTURAL_RECORD_ID`), where identity also depends on the name -- mirror
    // `NominalType::same_nominal` and fold the name in.
    if n.id < 0 {
        out.push_str(&format!("_nm{}_{}", n.name.len(), n.name));
    }
    for (name, ty) in entries {
        out.push_str(&format!("_fld{}_{name}_{}", name.len(), type_key(ty)));
    }
    out
}

/// Instance symbol of an instance-method call. `type_args[0]` is the receiver
/// type, so the symbol is unique per receiver layout; the method name keeps
/// distinct methods apart. Derivable from types alone (no HIR program), so the
/// monomorphizer and the back end agree.
pub fn method_symbol(method: &str, type_args: &[Type]) -> String {
    instance_symbol(&format!("{SYNTH_SIGIL}m_{method}"), type_args)
}

/// Instance symbol of a static call `Type.method(args)`.
///
/// A *no-argument* static that returns an aggregate (record/sum/array) is
/// return-polymorphic: a witness-free `HashMap.new()` whose element types are
/// fixed only by the caller. Its arguments alone do not distinguish
/// `HashMap<string,int32>` from `HashMap<int32,int32>`, so the result type is
/// folded into the key for those, giving each a distinct instance. Statics with
/// arguments are keyed by their arguments alone (the result is a function of
/// them), so non-generic statics are unaffected. The monomorphizer passes the
/// resolved result here; both back ends pass the call's destination type, which
/// is the same value, so all three derive the identical symbol.
pub fn static_symbol(ty: &str, method: &str, type_args: &[Type], result: Option<&Type>) -> String {
    let mut key = type_args.to_vec();
    if type_args.is_empty()
        && let Some(r) = result
        && is_return_polymorphic_result(r)
    {
        key.push(r.clone());
    }
    // The sigil also separates the type from the method: a type symbol may
    // itself contain `_` (`A_B.get` vs `A.B_get` must not share a symbol), and
    // no identifier can contain the sigil.
    instance_symbol(&format!("{SYNTH_SIGIL}s_{ty}{SYNTH_SIGIL}{method}"), &key)
}

/// Whether a no-argument static's result type can vary by caller and so must be
/// folded into its instance key: a record/sum built around inferred field types
/// (a witness-free constructor). Scalars/strings/void/arrays are left out,
/// keeping their symbols unchanged. Matches the seeding filter so the
/// monomorphizer and both back ends key these constructors identically.
pub(super) fn is_return_polymorphic_result(ty: &Type) -> bool {
    matches!(ty, Type::Record(_) | Type::Sum(_))
}

/// The monomorphized instance symbol for a `recv.name(args)` call that resolves
/// to a stdlib primitive/array method, when such an instance exists in
/// `program`. The receiver's class ([`Type::primitive_class`]) plus the method
/// name reconstruct the body's class-qualified symbol (the same scheme HIR
/// lowering used), which `instance_symbol` keys by argument types. Lets the back
/// ends route the call without carrying the HIR `primitive_methods` table.
pub fn prim_method_instance(
    program: &MonoProgram,
    name: &str,
    arg_types: &[Type],
) -> Option<String> {
    let class = arg_types.first()?.primitive_class()?;
    let sym = instance_symbol(&prepoly_hir::prim_method_symbol(class, name), arg_types);
    program.lookup(&sym).map(|_| sym)
}

/// Instance symbol of a closure: distinct per closure id, captured types, and
/// parameter types. Derivable from types alone so the monomorphizer and back end
/// agree.
pub fn closure_symbol(id: ClosureId, capture_types: &[Type], param_types: &[Type]) -> String {
    let mut args = capture_types.to_vec();
    args.extend_from_slice(param_types);
    instance_symbol(&format!("{SYNTH_SIGIL}clo{}", id.index()), &args)
}
