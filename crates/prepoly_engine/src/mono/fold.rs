//! Static-truthiness branch folding: deciding at monomorphization
//! time which conditional edges are dead (e.g. null checks on a
//! non-nullable local) so unreachable blocks are never compiled.

use super::*;

/// The blocks reachable from the entry once statically-known `if` conditions are
/// folded: a `never?` condition (a bare `null`, always null) is taken as false
/// and a non-nullable / non-bool condition (always truthy) as true, so the dead
/// arm is never visited. The typed back end uses this to skip emitting an arm
/// that cannot run -- and the unwrapped `never` values it would otherwise
/// contain (e.g. `a * 2` where `a` is a bare `null`) -- while monomorphization
/// still types both arms so a fallible callable's `Result` payloads infer from
/// whichever arm supplies each.
pub fn reachable_blocks(body: &MirBody, local_types: &[Type], ret: &Type) -> Vec<bool> {
    let mut reached = vec![false; body.blocks.len()];
    let mut stack = vec![body.entry];
    while let Some(id) = stack.pop() {
        if std::mem::replace(&mut reached[id.index()], true) {
            continue;
        }
        match &body.block(id).term {
            Terminator::Goto(b) => stack.push(*b),
            Terminator::CondBranch { cond, then, els } => {
                match cond_static_truthiness(body, local_types, ret, cond, *then) {
                    Some(true) => stack.push(*then),
                    Some(false) => stack.push(*els),
                    None => {
                        stack.push(*then);
                        stack.push(*els);
                    }
                }
            }
            Terminator::Return(_) | Terminator::Unreachable => {}
        }
    }
    reached
}

/// The effective static truthiness of an `if` condition, used to fold a branch.
/// Beyond the operand's own static truthiness, a *structural* `if` folds to false
/// when its then-branch cannot type for this concrete value: its reachable return
/// is a clear primitive-kind mismatch against the function's return type (a
/// structural field that is absent -- already `never?` -- or present at the wrong
/// type). The front end prunes the same dead arm; here the back end skips emitting
/// it so a generic function applied to a non-fitting structure degrades gracefully
/// (the guarded use is dead) rather than miscompiling.
pub fn cond_static_truthiness(
    body: &MirBody,
    local_types: &[Type],
    ret: &Type,
    cond: &Operand,
    then: BlockId,
) -> Option<bool> {
    if then_return_conflicts(body, local_types, ret, then) {
        return Some(false);
    }
    // An `if let x = e` presence test folds on the *subject's* nullability, not
    // its truthiness: any non-nullable subject -- including a bool -- is an
    // irrefutable bind (always then), a bare `null` is always absent, and only
    // a real nullable branches at runtime.
    if let Some(subj) = presence_subject_type(body, local_types, cond) {
        return match subj {
            Type::Nullable(inner) if matches!(**inner, Type::Never) => Some(false),
            Type::Nullable(_) | Type::Unknown(_) => None,
            _ => Some(true),
        };
    }
    operand_type_of(cond, local_types).static_truthiness()
}

/// If `cond` is the result of the `__present` builtin (the `if let` presence
/// test emitted by MIR lowering), the type of the subject it tests.
fn presence_subject_type<'t>(
    body: &MirBody,
    local_types: &'t [Type],
    cond: &Operand,
) -> Option<&'t Type> {
    let Operand::Local(id) = cond else {
        return None;
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            if let MirStmt::Assign(dest, Rvalue::Call(Callee::Builtin(name), args)) = stmt
                && dest == id
                && name == "__present"
                && let Some(Operand::Local(subj)) = args.first()
            {
                return Some(&local_types[subj.index()]);
            }
        }
    }
    None
}

/// Whether the then-branch reached unconditionally from `then` ends in a `return`
/// whose value's primitive kind clearly differs from `ret` (no coercion bridges
/// `string` vs `int`, etc.). A nested branch in the then-arm is not folded.
fn then_return_conflicts(body: &MirBody, local_types: &[Type], ret: &Type, then: BlockId) -> bool {
    // In a fallible callable a bare `return v` is the `Ok` payload, so compare the
    // returned value against the Ok payload type, not the whole `Result`.
    let target = match ret {
        Type::Sum(n) if n.id == RESULT_TYPE_ID => {
            n.result_payloads().map(|(ok, _)| ok).unwrap_or(ret)
        }
        _ => ret,
    };
    let mut id = then;
    let mut seen = HashSet::new();
    loop {
        if !seen.insert(id.index()) {
            return false;
        }
        match &body.block(id).term {
            Terminator::Return(op) => {
                let op_ty = operand_type_of(op, local_types);
                // A returned `Result` flows whole; only a bare value is the Ok
                // payload, so only it is compared against the Ok type.
                if matches!(&op_ty, Type::Sum(n) if n.id == RESULT_TYPE_ID) {
                    return false;
                }
                return primitive_kind_conflict(&op_ty, target);
            }
            Terminator::Goto(b) => id = *b,
            _ => return false,
        }
    }
}

/// Whether `a` and `b` are concrete primitives of different kinds (string/bool/
/// int/float), a mismatch no numeric conversion bridges. Non-primitive types are
/// treated as non-conflicting (conservative -- the fold only targets clear cases).
fn primitive_kind_conflict(a: &Type, b: &Type) -> bool {
    fn kind(t: &Type) -> Option<u8> {
        match t {
            Type::Str => Some(0),
            Type::Bool => Some(1),
            Type::Int(_) => Some(2),
            Type::Float(_) => Some(3),
            _ => None,
        }
    }
    matches!((kind(a), kind(b)), (Some(x), Some(y)) if x != y)
}
