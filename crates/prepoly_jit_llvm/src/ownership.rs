//! Automatic ownership analysis for `spawn` captures.
//!
//! A `spawn` runs its closure on a real OS thread, so each captured value is
//! shared between the spawner and that thread. The decision here is therefore
//! load-bearing, not advisory: [`auto_acquire`] realizes it before the spawn so
//! the capture has an atomic reference count from its first cross-thread
//! reference. A capture the closure mutates is made a cown (and its access wrapped
//! in `with`, which lock-guards it); a read-only capture is frozen (immutable).
//! Both are `rc_atomic` owner classes, which is what makes the otherwise-racy
//! cross-thread reference counting sound. [`decide`] additionally classifies a
//! capture as move/freeze/cown from its liveness for the auto-acquire diagnostic.

use std::collections::HashSet;

use prepoly_lexer::Span;
use prepoly_parser::ast::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownership {
    /// Not live after the spawn: hand exclusive ownership to the thread.
    Move,
    /// Live after the spawn but only read inside: freeze (immutable share).
    Freeze,
    /// Mutated inside the closure: wrap in a cown with auto-acquire.
    Cown,
}

/// Decide ownership for `var` captured by `closure_body`, given whether the
/// variable is still used after the spawn point.
pub fn decide(var: &str, live_after: bool, closure_body: &Block) -> Ownership {
    if !live_after {
        return Ownership::Move;
    }
    if mutates(closure_body, var) {
        Ownership::Cown
    } else {
        Ownership::Freeze
    }
}

/// Builtins known to only read their arguments, so passing a capture to one is
/// not a mutation. Everything else -- user functions and any other builtin -- is
/// treated conservatively as possibly mutating an argument it receives by place:
/// an unannotated parameter is a mutable reference by default (book: types.md), so
/// `f(var)` may write through `var`. Over-approximating mutation is the safe
/// direction -- it cowns (lock-guards) a capture rather than freezing it -- so a
/// genuinely read-only capture is at worst needlessly locked, never raced.
const READONLY_BUILTINS: &[&str] = &["println", "print"];

/// Whether `body` mutates `var`: assigns to it (or a field/element of it), calls a
/// method on it, or passes it (or a place rooted at it) to a function that is not
/// known read-only. The whole expression tree is traversed, so a mutation reached
/// through a `match`/`if let`/nested closure/array/record/interpolation is found
/// (a gap here would freeze a mutated capture and let two threads race it).
///
/// Simple local aliases are followed: a `let a = var` (or `let a = <place rooted
/// at a known alias>`) makes `a` another handle to the same object for the rest of
/// the body, so a later mutation through `a` counts as a mutation of `var`.
/// Without this, `let a = c; a.add(1)` inside a spawn closure would be rooted at
/// `a`, leave `c` classified read-only (frozen), and race it across threads.
pub fn mutates(body: &Block, var: &str) -> bool {
    let mut found = false;
    let mut aliases = HashSet::new();
    aliases.insert(var.to_string());
    scan_block(body, &mut aliases, &mut found);
    found
}

fn scan_block(b: &Block, aliases: &mut HashSet<String>, found: &mut bool) {
    for s in &b.stmts {
        if *found {
            return;
        }
        scan_stmt(s, aliases, found);
    }
}

fn scan_stmt(s: &Stmt, aliases: &mut HashSet<String>, found: &mut bool) {
    match s {
        Stmt::Let { pat, value, .. } => {
            scan_expr(value, aliases, found);
            // `let a = <place rooted at a current alias>` binds another handle to
            // the same shared object. Track `a` as an alias for the rest of the
            // scan. Only a plain place (ident / field / index) aliases the value;
            // a call or arithmetic produces a fresh value and is not tracked.
            if let Pattern::Binding(name, _) = pat
                && is_alias_place(value)
                && root_ident(value).is_some_and(|r| aliases.contains(r))
            {
                aliases.insert(name.clone());
            }
        }
        Stmt::Assign { target, value, .. } => {
            if root_ident(target).is_some_and(|r| aliases.contains(r)) {
                *found = true;
            }
            scan_expr(target, aliases, found);
            scan_expr(value, aliases, found);
        }
        Stmt::Expr(e) => scan_expr(e, aliases, found),
        Stmt::While { cond, body, .. } => {
            scan_expr(cond, aliases, found);
            scan_block(body, aliases, found);
        }
        Stmt::For { iter, body, .. } => {
            scan_expr(iter, aliases, found);
            scan_block(body, aliases, found);
        }
        Stmt::Return(Some(e), _) => scan_expr(e, aliases, found),
        Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

/// Whether `e` is a plain place expression (an identifier or a field/index
/// projection of one), so binding it aliases the same value rather than producing
/// a fresh one.
fn is_alias_place(e: &Expr) -> bool {
    match e {
        Expr::Ident(..) | Expr::SelfExpr(_) => true,
        Expr::Field(b, _, _) | Expr::Index(b, _, _) => is_alias_place(b),
        _ => false,
    }
}

/// Whether a call through `callee` is to a builtin known to only read its
/// arguments, so a capture passed to it is not mutated.
fn call_is_readonly(callee: &Expr) -> bool {
    matches!(callee, Expr::Ident(n, _) if READONLY_BUILTINS.contains(&n.as_str()))
}

/// Look for a mutation of any current alias inside an expression. Three forms
/// count as a mutation: a method call on a place rooted at an alias (`a.m(..)`),
/// an alias (or a place rooted at it) passed as an argument to a non-read-only
/// call, and -- via `scan_stmt` -- an assignment to it. Every sub-expression is
/// visited so a mutation nested anywhere is caught.
fn scan_expr(e: &Expr, aliases: &mut HashSet<String>, found: &mut bool) {
    if *found {
        return;
    }
    match e {
        Expr::Call(callee, args, _) => {
            // A method call on a place rooted at an alias may mutate it.
            if let Expr::Field(base, _, _) = &**callee
                && root_ident(base).is_some_and(|r| aliases.contains(r))
            {
                *found = true;
            }
            // Passing an alias (or `a.f` / `a[i]`) by reference to a function that
            // is not known read-only may mutate it through a mutable-reference
            // parameter. A value *derived* from an alias (e.g. `a + 1`, whose root
            // is not an identifier) is a fresh value and cannot alias it.
            if !call_is_readonly(callee) {
                for a in args {
                    if root_ident(&a.expr).is_some_and(|r| aliases.contains(r)) {
                        *found = true;
                    }
                }
            }
            scan_expr(callee, aliases, found);
            args.iter().for_each(|a| scan_expr(&a.expr, aliases, found));
        }
        Expr::Field(b, _, _) | Expr::Unary(_, b, _) | Expr::ErrorProp(b, _) => {
            scan_expr(b, aliases, found)
        }
        Expr::Index(b, idx, _) | Expr::Range(b, idx, _) | Expr::Binary(_, b, idx, _) => {
            scan_expr(b, aliases, found);
            scan_expr(idx, aliases, found);
        }
        Expr::Closure(_, body, _) => scan_expr(body, aliases, found),
        Expr::Array(elems, _) => elems.iter().for_each(|el| scan_expr(el, aliases, found)),
        Expr::TypeLit(_, fields, _) | Expr::VariantLit(_, _, fields, _) => fields
            .iter()
            .for_each(|(_, v)| scan_expr(v, aliases, found)),
        Expr::Str(segs, _) => segs.iter().for_each(|seg| {
            if let StrSeg::Expr(e) = seg {
                scan_expr(e, aliases, found);
            }
        }),
        Expr::Block(block, _) => scan_block(block, aliases, found),
        Expr::If(c, t, e, _) => {
            scan_expr(c, aliases, found);
            scan_block(t, aliases, found);
            if let Some(e) = e {
                scan_expr(e, aliases, found);
            }
        }
        Expr::IfLet(_, scrut, t, e, _) => {
            scan_expr(scrut, aliases, found);
            scan_block(t, aliases, found);
            if let Some(e) = e {
                scan_expr(e, aliases, found);
            }
        }
        Expr::Match(scrut, arms, _) => {
            scan_expr(scrut, aliases, found);
            arms.iter()
                .for_each(|arm| scan_expr(&arm.body, aliases, found));
        }
        Expr::Int(..)
        | Expr::Float(..)
        | Expr::Bool(..)
        | Expr::Null(_)
        | Expr::Ident(..)
        | Expr::SelfExpr(_) => {}
    }
}

fn root_ident(e: &Expr) -> Option<&str> {
    match e {
        Expr::Ident(n, _) => Some(n),
        Expr::Field(b, _, _) | Expr::Index(b, _, _) => root_ident(b),
        _ => None,
    }
}

/// Names referenced (and thus captured) inside a spawn closure body. `bound` is
/// the closure's own params and top-level locals. Names a *nested* closure binds
/// are also excluded: they belong to the inner scope, so a nested spawn's loop
/// counter is not mistaken for a capture of this one.
pub fn captured(body: &Block, bound: &HashSet<String>) -> HashSet<String> {
    let mut refs = HashSet::new();
    crate::closure::idents_block(body, &mut refs);
    let mut nested = HashSet::new();
    collect_nested_closure_bindings(body, &mut nested);
    refs.into_iter()
        .filter(|r| !bound.contains(r) && !nested.contains(r))
        .collect()
}

/// Collect the params and locals bound inside any closure nested in `body`. Used
/// by [`captured`] to drop an inner closure's own bindings from the outer
/// closure's capture set. Reuses [`scan_block`]-style traversal via the
/// closure free-variable helper in `crate::closure`.
fn collect_nested_closure_bindings(body: &Block, out: &mut HashSet<String>) {
    crate::closure::each_nested_closure(body, |params, cbody| {
        for p in params {
            out.insert(p.name.clone());
        }
        collect_local_bindings(&cbody.stmts, out);
    });
}

/// One captured variable's automatically chosen ownership at a `spawn` site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureDecision {
    pub var: String,
    pub ownership: Ownership,
}

/// Decide auto-ownership for every `spawn(closure)` capture reachable in a
/// function `body`. Liveness is approximated per
/// statement list: a capture is "live after" the spawn when it is referenced in
/// a later statement of the same block, so a value used only by the spawned
/// task is moved, one read afterwards is frozen, and one mutated inside the
/// closure and read afterwards is cowned. Over-approximating liveness is safe:
/// it can only upgrade `Move` to `Freeze`/`Cown`, never the reverse, which never
/// hands shared state to a thread that another path still uses. The decisions
/// are deterministic (sorted by variable name) so callers can report them.
pub fn analyze_spawns(body: &Block, params: &HashSet<String>) -> Vec<CaptureDecision> {
    analyze_spawns_stmts(&body.stmts, params)
}

/// As [`analyze_spawns`] but over a bare statement slice, for module-init code
/// whose top-level statements have no enclosing block. `params` are local names
/// already in scope (function parameters); local `let`/`for` bindings are added
/// automatically. Only these locals are subject to ownership transfer: a free
/// name that is a function or global is not a captured value.
pub fn analyze_spawns_stmts(stmts: &[Stmt], params: &HashSet<String>) -> Vec<CaptureDecision> {
    let mut locals = params.clone();
    collect_local_bindings(stmts, &mut locals);
    let mut out = Vec::new();
    analyze_block(stmts, &locals, &mut out);
    out
}

fn analyze_block(stmts: &[Stmt], locals: &HashSet<String>, out: &mut Vec<CaptureDecision>) {
    for (i, stmt) in stmts.iter().enumerate() {
        // Descend into every nested block (loops, conditionals, match arms, block
        // exprs), which may contain their own spawns.
        nested_block_stmts(stmt, &mut |inner| analyze_block(inner, locals, out));
        let Some(closure_body) = spawn_closure_body(stmt) else {
            continue;
        };
        // A nested spawn inside the spawned closure is its own site.
        analyze_block(&closure_body.stmts, locals, out);
        let bound = closure_bound_in(stmt);
        let mut captures: Vec<String> = captured(&closure_body, &bound)
            .into_iter()
            .filter(|name| locals.contains(name))
            .collect();
        captures.sort();
        let rest = &stmts[i + 1..];
        for var in captures {
            let live_after = stmts_reference(rest, &var);
            out.push(CaptureDecision {
                ownership: decide(&var, live_after, &closure_body),
                var,
            });
        }
    }
}

/// Collect names bound by `let`/`for` anywhere in a statement slice. Scoping is
/// ignored (a superset is safe: it only widens the set of names treated as
/// captured locals rather than globals). Every nested block -- loop bodies, `if`
/// / `if let` branches, `match` arms, block expressions -- is descended into, so a
/// capture bound inside one is still recognised as a transferable local.
fn collect_local_bindings(stmts: &[Stmt], out: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { pat, .. } => collect_pattern_bindings(pat, out),
            Stmt::For { var, .. } => {
                out.insert(var.clone());
            }
            _ => {}
        }
        nested_block_stmts(stmt, &mut |inner| collect_local_bindings(inner, out));
    }
}

/// Apply `f` to every statement list directly nested in `stmt` through control
/// flow: loop bodies, `if`/`if let` branches, `match` arms (block bodies), and
/// block expressions. Spawn closures are *not* descended into here (a spawn inside
/// a spawned closure is a separate site). The mutable twin is
/// [`nested_block_stmts_mut`].
fn nested_block_stmts(stmt: &Stmt, f: &mut dyn FnMut(&[Stmt])) {
    match stmt {
        Stmt::While { body, .. } | Stmt::For { body, .. } => f(&body.stmts),
        Stmt::Expr(e) | Stmt::Let { value: e, .. } | Stmt::Return(Some(e), _) => {
            nested_block_stmts_expr(e, f)
        }
        _ => {}
    }
}

fn nested_block_stmts_expr(e: &Expr, f: &mut dyn FnMut(&[Stmt])) {
    match e {
        Expr::If(_, t, els, _) | Expr::IfLet(_, _, t, els, _) => {
            f(&t.stmts);
            if let Some(els) = els {
                nested_block_stmts_expr(els, f);
            }
        }
        Expr::Block(b, _) => f(&b.stmts),
        Expr::Match(_, arms, _) => {
            for arm in arms {
                if let Expr::Block(b, _) = &arm.body {
                    f(&b.stmts);
                }
            }
        }
        _ => {}
    }
}

/// Mutable [`nested_block_stmts`]: apply `f` to every directly-nested statement
/// list so a pass that rewrites the AST (auto-acquire) reaches spawns nested in
/// conditionals and blocks, not only at the top level of a function or loop.
fn nested_block_stmts_mut(stmt: &mut Stmt, f: &mut dyn FnMut(&mut Vec<Stmt>)) {
    match stmt {
        Stmt::While { body, .. } | Stmt::For { body, .. } => f(&mut body.stmts),
        Stmt::Expr(e) | Stmt::Let { value: e, .. } | Stmt::Return(Some(e), _) => {
            nested_block_stmts_expr_mut(e, f)
        }
        _ => {}
    }
}

fn nested_block_stmts_expr_mut(e: &mut Expr, f: &mut dyn FnMut(&mut Vec<Stmt>)) {
    match e {
        Expr::If(_, t, els, _) | Expr::IfLet(_, _, t, els, _) => {
            f(&mut t.stmts);
            if let Some(els) = els {
                nested_block_stmts_expr_mut(els, f);
            }
        }
        Expr::Block(b, _) => f(&mut b.stmts),
        Expr::Match(_, arms, _) => {
            for arm in arms {
                if let Expr::Block(b, _) = &mut arm.body {
                    f(&mut b.stmts);
                }
            }
        }
        _ => {}
    }
}

fn collect_pattern_bindings(pat: &Pattern, out: &mut HashSet<String>) {
    match pat {
        Pattern::Binding(name, _) => {
            out.insert(name.clone());
        }
        Pattern::Record(_, fields, _) => {
            for field in fields {
                match &field.pat {
                    Some(sub) => collect_pattern_bindings(sub, out),
                    None => {
                        out.insert(field.name.clone());
                    }
                }
            }
        }
        Pattern::Array(pats, _) => pats.iter().for_each(|p| collect_pattern_bindings(p, out)),
        Pattern::Wildcard(_) | Pattern::Literal(_, _) => {}
    }
}

/// If `stmt` is a top-level `spawn(closure)` call (as an expression statement or
/// a `let` initializer), return the closure body as a block.
fn spawn_closure_body(stmt: &Stmt) -> Option<Block> {
    let expr = match stmt {
        Stmt::Expr(e) => e,
        Stmt::Let { value, .. } => value,
        _ => return None,
    };
    let Expr::Call(callee, args, _) = expr else {
        return None;
    };
    if !matches!(&**callee, Expr::Ident(n, _) if n == "spawn") {
        return None;
    }
    let Expr::Closure(_, body, _) = &args.first()?.expr else {
        return None;
    };
    Some(closure_block(body))
}

fn closure_bound_in(stmt: &Stmt) -> HashSet<String> {
    let expr = match stmt {
        Stmt::Expr(e) | Stmt::Let { value: e, .. } => e,
        _ => return HashSet::new(),
    };
    if let Expr::Call(_, args, _) = expr
        && let Some(Expr::Closure(params, body, _)) = args.first().map(|a| &a.expr)
    {
        return crate::closure::bound_names(params, &closure_block(body));
    }
    HashSet::new()
}

fn closure_block(body: &Expr) -> Block {
    match body {
        Expr::Block(b, _) => b.clone(),
        expr => Block {
            stmts: vec![Stmt::Expr(expr.clone())],
            span: expr.span(),
        },
    }
}

fn stmts_reference(stmts: &[Stmt], var: &str) -> bool {
    let mut refs = HashSet::new();
    crate::closure::idents_stmts(stmts, &mut refs);
    refs.contains(var)
}

// ----- auto-acquire -----

/// Insert automatic `with` acquisition for every `spawn` closure that mutates a
/// captured cown, so the programmer writes no ownership annotations yet shared
/// mutation is still serialized through the cown lock (wrap the
/// whole closure body in one `with` per cowned capture). A capture that is moved
/// (exclusive to the thread) or frozen (read-only) needs no lock and is left
/// alone. `params` are the names already in scope around `stmts`.
pub fn auto_acquire(stmts: &mut Vec<Stmt>, params: &HashSet<String>) {
    let mut locals = params.clone();
    collect_local_bindings(stmts, &mut locals);
    auto_acquire_in(stmts, &locals);
}

fn auto_acquire_in(stmts: &mut Vec<Stmt>, locals: &HashSet<String>) {
    // Cowns promoted by spawns already seen in this statement list, together with
    // any local aliases of them. Once a value is shared with a thread as a cown,
    // the spawner's *own* later accesses race the thread unless they too go through
    // the lock, so every later statement is guarded against this growing set. The
    // set includes aliases (`let a = counter`) because an access through a
    // different handle reaches the same object.
    let mut live_cowns: Vec<String> = Vec::new();
    let mut i = 0;
    while i < stmts.len() {
        // Classify this spawn's captures from the *pristine* body, before any
        // recursion rewrites it. Recursion may wrap a nested spawn's body in
        // `with(c, (c) -> ...)`, whose shadowing parameter would otherwise hide a
        // capture `c` from this outer spawn's analysis and skip promoting it.
        let (cowns, freezes) = spawn_capture_promotions(&stmts[i], locals);
        // Recurse into every nested block (loops, conditionals, match arms, block
        // exprs), which may themselves contain spawns to promote.
        nested_block_stmts_mut(&mut stmts[i], &mut |inner| auto_acquire_in(inner, locals));
        // Recurse into a spawn closure's own body so a nested spawn (spawn inside
        // spawn) has its captures promoted and its body guarded as its own site.
        if let Some(inner) = spawn_closure_body_mut(&mut stmts[i]) {
            auto_acquire_in(inner, locals);
        }
        if cowns.is_empty() && freezes.is_empty() {
            // Not a spawn site: guard the spawner's own access to any live cown (it
            // runs concurrently with the spawned task). Aliases were already folded
            // into `live_cowns` when the cown was promoted, so an access through a
            // different handle is covered too.
            if !live_cowns.is_empty() {
                guard_stmt_accesses(&mut stmts[i], &live_cowns);
            }
            i += 1;
            continue;
        }
        // Lock-guard each mutated capture's access inside the closure body.
        if !cowns.is_empty() {
            wrap_spawn_body(&mut stmts[i], &cowns);
        }
        // Promote every capture to an atomic-count owner *before* the spawn, so the
        // owner is fixed before the closure captures (retains) the value and the
        // count is atomic from the first cross-thread reference: a mutated capture
        // becomes a cown, a read-only one is frozen.
        let span = spawn_stmt_span(&stmts[i]);
        let promos: Vec<Stmt> = cowns
            .iter()
            .map(|c| promote_stmt("_cown", c, span))
            .chain(freezes.iter().map(|c| promote_stmt("_freeze", c, span)))
            .collect();
        let inserted = promos.len();
        for (k, p) in promos.into_iter().enumerate() {
            stmts.insert(i + k, p);
        }
        // The mutated captures -- and every local handle that aliases one of them
        // anywhere in this statement list -- are now live cowns for the spawner's
        // following statements. Aliases are collected over the whole list (not just
        // statements after the spawn) so a handle bound *before* the spawn, e.g.
        // `let a = counter; spawn(.. counter ..); a.add(..)`, is guarded too.
        for c in &cowns {
            for h in aliases_of(stmts, c) {
                if !live_cowns.contains(&h) {
                    live_cowns.push(h);
                }
            }
        }
        live_cowns.sort();
        i += inserted + 1;
    }
}

/// Every local handle that aliases `cown` within `stmts`: `cown` itself plus the
/// transitive closure of `let x = <a known handle>` bindings. Binding a bare cown
/// identifier copies the pointer (not itself a race), but a later access through
/// the new handle reaches the same object and must be guarded. Computed as a
/// fixpoint so a chain `let a = c; let b = a` includes `b`.
fn aliases_of(stmts: &[Stmt], cown: &str) -> Vec<String> {
    let mut handles: Vec<String> = vec![cown.to_string()];
    loop {
        let mut grew = false;
        collect_alias_bindings(stmts, &mut handles, &mut grew);
        if !grew {
            break;
        }
    }
    handles
}

fn collect_alias_bindings(stmts: &[Stmt], handles: &mut Vec<String>, grew: &mut bool) {
    for stmt in stmts {
        if let Stmt::Let {
            pat: Pattern::Binding(name, _),
            value: Expr::Ident(src, _),
            ..
        } = stmt
            && handles.iter().any(|h| h == src)
            && !handles.iter().any(|h| h == name)
        {
            handles.push(name.clone());
            *grew = true;
        }
        // Aliases may be bound inside nested blocks (a conditional, a loop) before
        // a use in the same scope; descend so they are found.
        nested_block_stmts(stmt, &mut |inner| {
            collect_alias_bindings(inner, handles, grew)
        });
    }
}

/// The mutable statement list of a `spawn(closure)`'s body in `stmt`, if `stmt`
/// is a spawn site whose argument is a block closure. Lets the transform recurse
/// into a nested spawn so its captures are promoted and its body guarded.
fn spawn_closure_body_mut(stmt: &mut Stmt) -> Option<&mut Vec<Stmt>> {
    let expr = match stmt {
        Stmt::Expr(e) | Stmt::Let { value: e, .. } => e,
        _ => return None,
    };
    let Expr::Call(callee, args, _) = expr else {
        return None;
    };
    if !matches!(&**callee, Expr::Ident(n, _) if n == "spawn") {
        return None;
    }
    let Expr::Closure(_, body, _) = &mut args.first_mut()?.expr else {
        return None;
    };
    match body.as_mut() {
        Expr::Block(b, _) => Some(&mut b.stmts),
        _ => None,
    }
}

/// Wrap each access to a live cown inside `stmt` in `with(c, (c) -> access)`, so
/// the spawner's own post-spawn use of a shared cown is serialized through the
/// same lock the spawned task uses. The reentrant lock makes nested guards on the
/// same cown safe. A statement that touches no live cown is left unchanged.
fn guard_stmt_accesses(stmt: &mut Stmt, cowns: &[String]) {
    match stmt {
        Stmt::Let { value, .. } => guard_expr_accesses(value, cowns),
        Stmt::Assign { target, value, .. } => {
            // A whole `c.f = v` (or `c[i] = v`) store is guarded as one unit so
            // both the place and the value evaluate under the lock.
            if root_ident(target).is_some_and(|r| cowns.contains(&r.to_string())) {
                guard_assign(stmt);
                return;
            }
            guard_expr_accesses(target, cowns);
            guard_expr_accesses(value, cowns);
        }
        Stmt::Expr(e) | Stmt::Return(Some(e), _) => guard_expr_accesses(e, cowns),
        Stmt::While { cond, body, .. } => {
            guard_expr_accesses(cond, cowns);
            for s in &mut body.stmts {
                guard_stmt_accesses(s, cowns);
            }
        }
        Stmt::For { iter, body, .. } => {
            guard_expr_accesses(iter, cowns);
            for s in &mut body.stmts {
                guard_stmt_accesses(s, cowns);
            }
        }
        Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

/// Rewrite a field/element-store statement `c.f = v` whose root is a live cown
/// into `with(c, (c) -> { c.f = v })`, so the read-modify-write happens under the
/// lock. The store is moved into the `with` closure body verbatim.
fn guard_assign(stmt: &mut Stmt) {
    let placeholder = Stmt::Break(stmt_span(stmt));
    let original = std::mem::replace(stmt, placeholder);
    let Stmt::Assign { target, .. } = &original else {
        *stmt = original;
        return;
    };
    let Some(root) = root_ident(target).map(str::to_string) else {
        *stmt = original;
        return;
    };
    let span = stmt_span(&original);
    let block = Block {
        stmts: vec![original],
        span,
    };
    *stmt = Stmt::Expr(wrap_with(Expr::Block(block, span), &[root]));
}

/// Wrap each cown access within an expression in a `with`. An expression node
/// that *directly* dereferences a live cown -- a method call `c.m(..)`, a
/// field/index read `c.f` / `c[i]`, or a call `f(.., c, ..)` that forwards the
/// cown (or a place rooted at it) by reference -- is wrapped whole so its whole
/// evaluation holds the cown's lock. Sub-expressions are guarded first, so a
/// nested access (a cown read inside an argument, branch, or interpolation) is
/// also serialized; the reentrant lock makes the resulting nested `with`s on the
/// same cown safe.
fn guard_expr_accesses(expr: &mut Expr, cowns: &[String]) {
    // Guard nested accesses first so they are serialized even when this node is
    // not itself wrapped (and stay correctly nested when it is).
    guard_children(expr, cowns);
    // Then, if this node directly dereferences live cowns, wrap the whole node.
    let roots = direct_cown_roots(expr, cowns);
    if !roots.is_empty() {
        let span = expr.span();
        let original = std::mem::replace(expr, Expr::Null(span));
        *expr = wrap_with(original, &roots);
    }
}

/// Recurse into an expression's children, guarding any cown access inside them.
/// A `Call`'s by-reference operands (its method receiver and the arguments that
/// are plain places rooted at a cown) are *not* descended into here -- they are
/// handled by wrapping the whole call in [`guard_expr_accesses`] -- so a bare cown
/// argument is not mistaken for a standalone read.
fn guard_children(expr: &mut Expr, cowns: &[String]) {
    match expr {
        Expr::Call(callee, args, _) => {
            // A method receiver is part of the call's by-reference access; only
            // descend into a non-method callee.
            if !matches!(callee.as_ref(), Expr::Field(..)) {
                guard_expr_accesses(callee, cowns);
            }
            for a in args.iter_mut() {
                // A bare cown place passed as an argument is guarded by wrapping
                // the call; descend only into compound argument expressions.
                if direct_arg_cown_root(&a.expr, cowns).is_none() {
                    guard_expr_accesses(&mut a.expr, cowns);
                }
            }
        }
        Expr::Field(b, _, _) | Expr::Unary(_, b, _) | Expr::ErrorProp(b, _) => {
            guard_expr_accesses(b, cowns)
        }
        Expr::Index(b, idx, _) | Expr::Range(b, idx, _) | Expr::Binary(_, b, idx, _) => {
            guard_expr_accesses(b, cowns);
            guard_expr_accesses(idx, cowns);
        }
        Expr::Array(elems, _) => elems
            .iter_mut()
            .for_each(|el| guard_expr_accesses(el, cowns)),
        Expr::TypeLit(_, fields, _) | Expr::VariantLit(_, _, fields, _) => fields
            .iter_mut()
            .for_each(|(_, v)| guard_expr_accesses(v, cowns)),
        Expr::Str(segs, _) => segs.iter_mut().for_each(|seg| {
            if let StrSeg::Expr(e) = seg {
                guard_expr_accesses(e, cowns);
            }
        }),
        Expr::Block(block, _) => {
            for s in &mut block.stmts {
                guard_stmt_accesses(s, cowns);
            }
        }
        Expr::If(c, t, e, _) => {
            guard_expr_accesses(c, cowns);
            for s in &mut t.stmts {
                guard_stmt_accesses(s, cowns);
            }
            if let Some(e) = e {
                guard_expr_accesses(e, cowns);
            }
        }
        Expr::IfLet(_, scrut, t, e, _) => {
            guard_expr_accesses(scrut, cowns);
            for s in &mut t.stmts {
                guard_stmt_accesses(s, cowns);
            }
            if let Some(e) = e {
                guard_expr_accesses(e, cowns);
            }
        }
        Expr::Match(scrut, arms, _) => {
            guard_expr_accesses(scrut, cowns);
            arms.iter_mut()
                .for_each(|arm| guard_expr_accesses(&mut arm.body, cowns));
        }
        // A closure here is a non-spawn closure value; its body is not part of the
        // spawner's straight-line flow, so leave it (a spawn closure was already
        // handled at the spawn site).
        Expr::Closure(..)
        | Expr::Int(..)
        | Expr::Float(..)
        | Expr::Bool(..)
        | Expr::Null(_)
        | Expr::Ident(..)
        | Expr::SelfExpr(_) => {}
    }
}

/// The live-cown roots this expression node *directly* dereferences (sorted,
/// deduplicated). A method call contributes its receiver root and every argument
/// rooted at a cown (each forwarded by reference); a plain call contributes its
/// argument roots; a field/index read contributes its base root. A bare
/// identifier contributes nothing -- copying a cown handle does not dereference
/// the object.
fn direct_cown_roots(expr: &Expr, cowns: &[String]) -> Vec<String> {
    let mut roots = Vec::new();
    let mut push = |r: Option<String>| {
        if let Some(r) = r
            && !roots.contains(&r)
        {
            roots.push(r);
        }
    };
    match expr {
        Expr::Call(callee, args, _) => {
            if let Expr::Field(base, _, _) = callee.as_ref() {
                push(cown_root(base, cowns));
            }
            for a in args {
                push(direct_arg_cown_root(&a.expr, cowns));
            }
        }
        Expr::Field(base, _, _) | Expr::Index(base, _, _) => push(cown_root(base, cowns)),
        _ => {}
    }
    roots.sort();
    roots
}

/// The cown root of a call argument that is a plain place (`c`, `c.f`, `c[i]`)
/// forwarded by reference, if it is a live cown. A compound argument (a call,
/// arithmetic) produces a fresh value and is not a by-reference forward.
fn direct_arg_cown_root(arg: &Expr, cowns: &[String]) -> Option<String> {
    match arg {
        Expr::Ident(..) | Expr::SelfExpr(_) | Expr::Field(..) | Expr::Index(..) => {
            cown_root(arg, cowns)
        }
        _ => None,
    }
}

/// The root identifier of a place, when it is one of the live cowns.
fn cown_root(place: &Expr, cowns: &[String]) -> Option<String> {
    root_ident(place)
        .filter(|r| cowns.contains(&r.to_string()))
        .map(str::to_string)
}

/// The span of a statement, for synthesizing a wrapping `with`.
fn stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Let { value, .. } => value.span(),
        Stmt::Assign { span, .. } => *span,
        Stmt::Expr(e) | Stmt::Return(Some(e), _) => e.span(),
        Stmt::While { cond, .. } => cond.span(),
        Stmt::For { iter, .. } => iter.span(),
        Stmt::Return(None, span) | Stmt::Break(span) | Stmt::Continue(span) => *span,
    }
}

/// The span used for a spawn's synthesized promotion statements (cosmetic: these
/// builtins do not fail, so the span only ever surfaces in internal diagnostics).
fn spawn_stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Expr(e) | Stmt::Let { value: e, .. } => e.span(),
        _ => Span::new(0, 0),
    }
}

/// A `builtin(var)` statement -- `_cown(var)` or `_freeze(var)` -- inserted before
/// a spawn to promote `var` to an atomic-count owner.
fn promote_stmt(builtin: &str, var: &str, span: Span) -> Stmt {
    Stmt::Expr(Expr::Call(
        Box::new(Expr::Ident(builtin.to_string(), span)),
        vec![Arg {
            expr: Expr::Ident(var.to_string(), span),
        }],
        span,
    ))
}

/// The captures of a `spawn` closure in `stmt`, partitioned into `(cowns,
/// freezes)`: those the closure mutates (cowned, so their access is lock-guarded)
/// and those it only reads (frozen). Every captured local crosses to the new
/// thread, so both groups are promoted to an atomic-count owner -- the move/freeze
/// liveness distinction is not used for atomicity, only the mutation does. Sorted
/// for deterministic, deadlock-free lock ordering.
fn spawn_capture_promotions(stmt: &Stmt, locals: &HashSet<String>) -> (Vec<String>, Vec<String>) {
    let Some(closure_body) = spawn_closure_body(stmt) else {
        return (Vec::new(), Vec::new());
    };
    let bound = closure_bound_in(stmt);
    let mut caps: Vec<String> = captured(&closure_body, &bound)
        .into_iter()
        .filter(|name| locals.contains(name))
        .collect();
    caps.sort();
    caps.into_iter()
        .partition(|var| mutates(&closure_body, var))
}

/// Rewrite the spawn closure's body in `stmt` to `with(c, (c) -> body)` nested
/// per cown, so each cowned capture is acquired around the body.
fn wrap_spawn_body(stmt: &mut Stmt, cowns: &[String]) {
    let expr = match stmt {
        Stmt::Expr(e) | Stmt::Let { value: e, .. } => e,
        _ => return,
    };
    let Expr::Call(callee, args, _) = expr else {
        return;
    };
    if !matches!(&**callee, Expr::Ident(n, _) if n == "spawn") {
        return;
    }
    let Some(arg) = args.first_mut() else { return };
    let Expr::Closure(_, body, span) = &mut arg.expr else {
        return;
    };
    let span = *span;
    let original = std::mem::replace(body.as_mut(), Expr::Null(span));
    *body.as_mut() = wrap_with(original, cowns);
}

/// Build `with(c0, (c0) -> with(c1, (c1) -> body))` for cowns `[c0, c1]`. The
/// `with` handle shadows the captured name so the body's accesses go through it.
fn wrap_with(body: Expr, cowns: &[String]) -> Expr {
    let mut inner = body;
    for cown in cowns.iter().rev() {
        let span = inner.span();
        let param = Param {
            name: cown.clone(),
            ty: None,
            span,
        };
        let closure = Expr::Closure(vec![param], Box::new(inner), span);
        inner = Expr::Call(
            Box::new(Expr::Ident("with".to_string(), span)),
            vec![
                Arg {
                    expr: Expr::Ident(cown.clone(), span),
                },
                Arg { expr: closure },
            ],
            span,
        );
    }
    inner
}

#[cfg(test)]
mod tests {
    use super::*;
    use prepoly_parser::ast::TopLevel;
    use prepoly_parser::parse;

    fn main_body(src: &str) -> Block {
        let module = parse(src).expect("parse");
        for item in module.items {
            if let TopLevel::Fun(f) = item
                && f.name == "main"
            {
                return f.body;
            }
        }
        panic!("no `main`");
    }

    fn decisions(src: &str) -> Vec<CaptureDecision> {
        analyze_spawns(&main_body(src), &HashSet::new())
    }

    #[test]
    fn capture_only_used_by_spawn_is_moved() {
        let d = decisions(
            "fun main() {\n    let data = [1, 2, 3]\n    spawn(() -> { println(data) })\n}\n",
        );
        assert_eq!(
            d,
            vec![CaptureDecision {
                var: "data".into(),
                ownership: Ownership::Move
            }]
        );
    }

    #[test]
    fn capture_read_after_spawn_is_frozen() {
        let d = decisions(
            "fun main() {\n    let data = [1, 2, 3]\n    spawn(() -> { println(data) })\n    println(data)\n}\n",
        );
        assert_eq!(
            d,
            vec![CaptureDecision {
                var: "data".into(),
                ownership: Ownership::Freeze
            }]
        );
    }

    #[test]
    fn capture_mutated_in_spawn_and_live_is_cowned() {
        let d = decisions(
            "fun main() {\n    let count = 0\n    spawn(() -> { count = count + 1 })\n    println(count)\n}\n",
        );
        assert_eq!(
            d,
            vec![CaptureDecision {
                var: "count".into(),
                ownership: Ownership::Cown
            }]
        );
    }

    #[test]
    fn no_spawn_yields_no_decisions() {
        assert!(decisions("fun main() {\n    let x = 1\n    println(x)\n}\n").is_empty());
    }

    #[test]
    fn method_call_on_capture_counts_as_mutation() {
        // A mutating method call (not a direct assignment) must still cown the
        // capture, so it is lock-guarded across threads.
        let d = decisions(
            "fun main() {\n    let c = make()\n    spawn(() -> { c.add(1) })\n    use(c)\n}\n",
        );
        assert_eq!(
            d,
            vec![CaptureDecision {
                var: "c".into(),
                ownership: Ownership::Cown
            }]
        );
    }

    #[test]
    fn mutation_through_a_ref_function_call_counts_as_mutation() {
        // Passing the capture to a function (whose default parameter is a mutable
        // reference) may mutate it, so it must be cowned -- not frozen, which would
        // let two threads race it. This is the form `ownership::mutates` previously
        // missed (it only caught `c.m()` and `c = ..`).
        let d = decisions(
            "fun main() {\n    let c = make()\n    spawn(() -> { bump(c) })\n    use(c)\n}\n",
        );
        assert_eq!(
            d,
            vec![CaptureDecision {
                var: "c".into(),
                ownership: Ownership::Cown
            }]
        );
    }

    #[test]
    fn mutation_inside_a_match_arm_counts_as_mutation() {
        // A mutation nested in a `match` arm body must be found by the full-tree
        // scan; missing it would freeze a mutated capture and race it.
        let d = decisions(
            "fun main() {\n    let c = make()\n    spawn(() -> { match 1 {\n        _ => { c.x = 1 }\n    } })\n    use(c)\n}\n",
        );
        assert_eq!(
            d,
            vec![CaptureDecision {
                var: "c".into(),
                ownership: Ownership::Cown
            }]
        );
    }

    #[test]
    fn read_only_call_keeps_capture_frozen() {
        // The read-only-builtin whitelist keeps a capture passed only to `println`
        // frozen rather than needlessly cowning it.
        let d = decisions(
            "fun main() {\n    let c = make()\n    spawn(() -> { println(c) })\n    use(c)\n}\n",
        );
        assert_eq!(
            d,
            vec![CaptureDecision {
                var: "c".into(),
                ownership: Ownership::Freeze
            }]
        );
    }

    #[test]
    fn auto_acquire_wraps_a_cowned_spawn_body_in_with() {
        // The cowned capture's spawn body is rewritten to acquire it via `with`,
        // so concurrent mutation goes through the lock without any annotation.
        let mut body = main_body(
            "fun main() {\n    let c = make()\n    spawn(() -> { c.add(1) })\n    use(c)\n}\n",
        );
        auto_acquire(&mut body.stmts, &HashSet::new());
        let wrapped = body.stmts.iter().any(spawn_body_is_with);
        assert!(wrapped, "cowned spawn body should be wrapped in `with`");
    }

    #[test]
    fn mutation_through_an_alias_in_the_closure_counts_as_mutation() {
        // A capture mutated through a local alias (`let a = c; a.add(1)`) is rooted
        // at `a`, but `a` aliases `c`, so the capture must still be cowned -- not
        // frozen, which would race it.
        let d = decisions(
            "fun main() {\n    let c = make()\n    spawn(() -> { let a = c\n        a.add(1) })\n    use(c)\n}\n",
        );
        assert_eq!(
            d,
            vec![CaptureDecision {
                var: "c".into(),
                ownership: Ownership::Cown
            }]
        );
    }

    #[test]
    fn nested_closure_local_is_not_captured_by_the_outer_spawn() {
        // A loop counter bound inside a nested spawn closure belongs to the inner
        // scope; it must not be seen as a capture (and promoted) by the outer spawn.
        let body = main_body(
            "fun main() {\n    let c = make()\n    spawn(() -> {\n        spawn(() -> { let i = 0\n            c.add(i) })\n    })\n}\n",
        );
        // The outer spawn's closure body is the first statement's argument.
        let Stmt::Expr(Expr::Call(_, args, _)) = &body.stmts[1] else {
            panic!("expected spawn statement");
        };
        let Expr::Closure(_, outer_body, _) = &args[0].expr else {
            panic!("expected closure");
        };
        let Expr::Block(b, _) = outer_body.as_ref() else {
            panic!("expected block body");
        };
        let caps = captured(b, &HashSet::new());
        assert!(
            !caps.contains("i"),
            "the inner closure's local `i` must not be a capture of the outer spawn: {caps:?}"
        );
        assert!(
            caps.contains("c"),
            "the outer spawn still captures `c`: {caps:?}"
        );
    }

    #[test]
    fn spawner_access_after_a_cowned_spawn_is_guarded() {
        // The spawner's own later access to a cowned capture must be wrapped in
        // `with` too, so it does not race the spawned task. Here the trailing
        // `c.add(2)` runs concurrently with the spawned `c.add(1)`.
        let mut body = main_body(
            "fun main() {\n    let c = make()\n    spawn(() -> { c.add(1) })\n    c.add(2)\n}\n",
        );
        auto_acquire(&mut body.stmts, &HashSet::new());
        // The last statement is the spawner's `c.add(2)`; after the transform it
        // must be a `with(c, ...)` guard.
        let guarded = body.stmts.iter().any(|s| {
            matches!(s, Stmt::Expr(Expr::Call(callee, _, _))
                if matches!(&**callee, Expr::Ident(n, _) if n == "with"))
        });
        assert!(
            guarded,
            "the spawner's own access to the cown should be guarded by `with`"
        );
    }

    fn spawn_body_is_with(stmt: &Stmt) -> bool {
        let Stmt::Expr(Expr::Call(callee, args, _)) = stmt else {
            return false;
        };
        if !matches!(&**callee, Expr::Ident(n, _) if n == "spawn") {
            return false;
        }
        let Some(Expr::Closure(_, body, _)) = args.first().map(|a| &a.expr) else {
            return false;
        };
        matches!(&**body, Expr::Call(c, _, _) if matches!(&**c, Expr::Ident(n, _) if n == "with"))
    }
}
