//! Record types crossing the JIT boundary: resolving a nominal or
//! structural (anonymous) record to the concrete field layout the
//! back end shares with the host, via the encoded field descriptor.

use super::*;

/// Build the concrete `Type` of a declared record as it arrives at the runtime
/// deserialize boundary: a nominal carrying every field's declared
/// type in its substitution, exactly as a constructed record does -- so it
/// satisfies the typed backend's support check and field reads resolve. Returns
/// `None` if `name` is not a record type in `module`, or a field type is unknown.
/// (A future structural deserializer builds the substitution from the data's
/// shape; for a declared target this derives it from the field declarations.)
pub fn boundary_record_type(program: &Program, module: &[String], name: &str) -> Option<Type> {
    boundary_record_type_of(program.resolve_type(module, name)?)
}

/// Like [`boundary_record_type`] but keyed by the type's id -- the tag a boundary
/// value carries at runtime. The dispatch trampoline rebuilds the consumer's
/// argument type from a runtime value's tag with this.
pub fn boundary_record_type_by_id(program: &Program, id: i32) -> Option<Type> {
    boundary_record_type_of(program.type_by_id(id)?)
}

/// Like [`boundary_record_type`] but found by the type's source name across all
/// modules (the deserialize boundary names its target type); the first match
/// wins. Used by the dispatch trampoline.
pub fn boundary_record_type_by_name(program: &Program, name: &str) -> Option<Type> {
    program
        .types
        .values()
        .find(|t| t.name == name)
        .and_then(boundary_record_type_of)
}

/// The sentinel type id for a *structural* record built at the deserialize boundary
/// from a value's shape rather than a declaration. No declared type
/// uses this id, so `type_by_id` misses and the typed backend lays the record out
/// from its substitution (sorted field order) instead of a declaration.
pub const STRUCTURAL_RECORD_ID: i32 = i32::MIN;

/// Build a `Type::Record` from a field list discovered at the deserialize
/// boundary: the data structure -- not a declared type name -- drives the
/// type. The resulting record has no declaration; its layout comes from the
/// substitution (the typed backend orders structural fields by name). The consumer
/// is then monomorphized against this type exactly like a declared one, and the
/// boundary's structural-requirement check rejects a value missing a read field.
pub fn boundary_record_type_from_fields(fields: &[(String, Type)]) -> Type {
    let mut subst = Substitution::empty();
    for (name, ty) in fields {
        subst.insert(name.clone(), ty.clone());
    }
    Type::Record(NominalType::with_substitution(
        STRUCTURAL_RECORD_ID,
        "<structural>",
        subst,
    ))
}

/// Parse a structural record descriptor `"field:tag,field:tag"` (optionally brace-
/// wrapped) into ordered `(field, Type)` pairs, the data-driven type description a
/// `deserialize` boundary produces. Returns `None` on a malformed
/// descriptor or an unknown field type tag.
pub fn parse_structural_descriptor(desc: &str) -> Option<Vec<(String, Type)>> {
    let body = desc
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if body.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for field in body.split(',') {
        let (name, tag) = field.split_once(':')?;
        out.push((name.trim().to_string(), type_from_tag(tag.trim())?));
    }
    Some(out)
}

/// The `Type` named by a structural-descriptor field tag.
fn type_from_tag(tag: &str) -> Option<Type> {
    if let Some(k) = IntKind::from_name(tag) {
        return Some(Type::Int(k));
    }
    Some(match tag {
        "float32" => Type::Float(FloatKind::F32),
        "float64" => Type::Float(FloatKind::F64),
        "string" => Type::Str,
        "bool" => Type::Bool,
        _ => return None,
    })
}

fn boundary_record_type_of(info: &prepoly_hir::TypeInfo) -> Option<Type> {
    let TypeKind::Record { fields, .. } = &info.kind else {
        return None;
    };
    let mut subst = Substitution::empty();
    for f in fields {
        subst.insert(f.name.clone(), f.resolved_ty.clone()?);
    }
    Some(Type::Record(NominalType::with_substitution(
        info.id,
        info.name.clone(),
        subst,
    )))
}
