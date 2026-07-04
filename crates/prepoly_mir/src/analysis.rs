//! Free-variable analysis over the AST, used to decide a closure's captures.
//!
//! This duplicates the small AST walk also used by codegen's closure conversion
//! (it cannot depend on `prepoly_jit_llvm`, which will in turn depend on this
//! crate). Given a closure's parameters and body, [`free_vars_of`] returns the
//! identifiers it references but does not itself bind; lowering keeps those that
//! are bound in the enclosing scope as the closure's captured operands.

use std::collections::HashSet;

use prepoly_parser::ast::*;

/// Locals that must be heap-promoted to a shared cell: those a
/// closure captures *and* that are assigned (mutated) somewhere -- in the closure or
/// the enclosing body -- so the mutation is observed through the shared capture, not
/// made on a per-closure copy. Read-only captures stay by-value. The cell is modeled
/// as a one-element array, reusing the array machinery (alloc, indexed get/set, RC).
pub fn cell_promotions(body: &Block) -> HashSet<String> {
    let mut captured = HashSet::new();
    let mut mutated = HashSet::new();
    promo_block(body, &mut captured, &mut mutated);
    captured.intersection(&mutated).cloned().collect()
}

fn promo_block(b: &Block, cap: &mut HashSet<String>, mutd: &mut HashSet<String>) {
    for s in &b.stmts {
        match s {
            Stmt::Let {
                value: Some(value), ..
            } => promo_expr(value, cap, mutd),
            Stmt::Let { value: None, .. } => {}
            Stmt::Assign { target, value, .. } => {
                if let Expr::Ident(n, _) = target {
                    mutd.insert(n.clone());
                }
                promo_expr(target, cap, mutd);
                promo_expr(value, cap, mutd);
            }
            Stmt::Expr(e) => promo_expr(e, cap, mutd),
            Stmt::While { cond, body, .. } => {
                promo_expr(cond, cap, mutd);
                promo_block(body, cap, mutd);
            }
            Stmt::For { iter, body, .. } => {
                promo_expr(iter, cap, mutd);
                promo_block(body, cap, mutd);
            }
            Stmt::Return(Some(e), _) => promo_expr(e, cap, mutd),
            _ => {}
        }
    }
}

fn promo_expr(e: &Expr, cap: &mut HashSet<String>, mutd: &mut HashSet<String>) {
    match e {
        Expr::Closure(params, cbody, _) => {
            let cb = closure_as_block(cbody);
            for v in free_vars_of(params, &cb) {
                cap.insert(v);
            }
            promo_block(&cb, cap, mutd);
        }
        Expr::Block(b, _) => promo_block(b, cap, mutd),
        Expr::If(c, t, els, _) => {
            promo_expr(c, cap, mutd);
            promo_block(t, cap, mutd);
            if let Some(e) = els {
                promo_expr(e, cap, mutd);
            }
        }
        Expr::IfLet(_, s, t, els, _) => {
            promo_expr(s, cap, mutd);
            promo_block(t, cap, mutd);
            if let Some(e) = els {
                promo_expr(e, cap, mutd);
            }
        }
        Expr::Match(s, arms, _) => {
            promo_expr(s, cap, mutd);
            for a in arms {
                promo_expr(&a.body, cap, mutd);
            }
        }
        _ => walk_subexprs(e, &mut |sub| promo_expr(sub, cap, mutd)),
    }
}

/// A closure body viewed as a block (an expression body becomes a one-statement
/// block). Mirrors `lower::closure_block`.
fn closure_as_block(body: &Expr) -> Block {
    match body {
        Expr::Block(b, _) => b.clone(),
        other => Block {
            stmts: vec![Stmt::Expr(other.clone())],
            span: other.span(),
        },
    }
}

/// Free variables of a closure body relative to its own parameters and locals,
/// sorted for deterministic capture order. The walk is ordered and scoped: a
/// read *before* a same-named later `let` is free (it refers to the outer
/// binding the closure must capture), and a binding introduced inside a nested
/// block, loop body, or match/if-let arm goes out of scope with it.
pub fn free_vars_of(params: &[Param], body: &Block) -> Vec<String> {
    let mut bound: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();
    let mut free = HashSet::new();
    free_block(body, &mut bound, &mut free);
    let mut out: Vec<String> = free.into_iter().collect();
    out.sort();
    out
}

/// Walk a block in statement order, adding reads of names not currently bound
/// to `free`. `bound` is restored on exit, so the block's `let`s do not leak
/// into the enclosing scope.
fn free_block(b: &Block, bound: &mut HashSet<String>, free: &mut HashSet<String>) {
    let saved = bound.clone();
    for s in &b.stmts {
        match s {
            Stmt::Let { pat, value, .. } => {
                // The initializer is evaluated before the pattern binds, so a
                // self-named reference in it (`let y = y + 1`) is a free read.
                if let Some(value) = value {
                    free_expr(value, bound, free);
                }
                bound_pat(pat, bound);
            }
            Stmt::Assign { target, value, .. } => {
                free_expr(target, bound, free);
                free_expr(value, bound, free);
            }
            Stmt::Expr(e) => free_expr(e, bound, free),
            Stmt::While { cond, body, .. } => {
                free_expr(cond, bound, free);
                free_block(body, bound, free);
            }
            Stmt::For {
                var, iter, body, ..
            } => {
                free_expr(iter, bound, free);
                let mut inner = bound.clone();
                inner.insert(var.clone());
                free_block(body, &mut inner, free);
            }
            Stmt::Return(Some(e), _) => free_expr(e, bound, free),
            _ => {}
        }
    }
    *bound = saved;
}

fn free_expr(e: &Expr, bound: &mut HashSet<String>, free: &mut HashSet<String>) {
    match e {
        Expr::Ident(n, _) => {
            if !bound.contains(n) {
                free.insert(n.clone());
            }
        }
        // A nested closure binds its own parameters; whatever else it reads and
        // we have not bound is transitively free here.
        Expr::Closure(params, body, _) => {
            let mut inner = bound.clone();
            for p in params {
                inner.insert(p.name.clone());
            }
            free_block(&closure_as_block(body), &mut inner, free);
        }
        Expr::Block(b, _) => free_block(b, bound, free),
        Expr::If(c, t, els, _) => {
            free_expr(c, bound, free);
            free_block(t, bound, free);
            if let Some(e) = els {
                free_expr(e, bound, free);
            }
        }
        // Pattern bindings are visible only in the arm they guard.
        Expr::IfLet(pat, s, t, els, _) => {
            free_expr(s, bound, free);
            let mut inner = bound.clone();
            bound_pat(pat, &mut inner);
            free_block(t, &mut inner, free);
            if let Some(e) = els {
                free_expr(e, bound, free);
            }
        }
        Expr::Match(s, arms, _) => {
            free_expr(s, bound, free);
            for a in arms {
                let mut inner = bound.clone();
                bound_pat(&a.pattern, &mut inner);
                free_expr(&a.body, &mut inner, free);
            }
        }
        _ => walk_subexprs(e, &mut |sub| free_expr(sub, bound, free)),
    }
}

fn bound_pat(p: &Pattern, out: &mut HashSet<String>) {
    match p {
        Pattern::Binding(n, _) => {
            out.insert(n.clone());
        }
        Pattern::Array(ps, _) => ps.iter().for_each(|p| bound_pat(p, out)),
        Pattern::Record(_, fps, _) => {
            for fp in fps {
                match &fp.pat {
                    Some(sub) => bound_pat(sub, out),
                    None => {
                        out.insert(fp.name.clone());
                    }
                }
            }
        }
        _ => {}
    }
}

/// Apply `f` to the immediate value sub-expressions of `e`. Nested closures are
/// walked through their body so transitively captured names are still seen.
fn walk_subexprs(e: &Expr, f: &mut impl FnMut(&Expr)) {
    match e {
        Expr::Unary(_, a, _) | Expr::Field(a, _, _) | Expr::ErrorProp(a, _) => f(a),
        Expr::Binary(_, a, b, _) | Expr::Index(a, b, _) | Expr::Range(a, b, _) => {
            f(a);
            f(b);
        }
        Expr::Call(c, args, _) => {
            f(c);
            for a in args {
                f(&a.expr);
            }
        }
        Expr::Closure(_, body, _) => f(body),
        Expr::Array(es, _) => es.iter().for_each(f),
        Expr::TypeLit(_, fs, _) | Expr::VariantLit(_, _, fs, _) => {
            fs.iter().for_each(|(_, e)| f(e))
        }
        Expr::Str(segs, _) => {
            for s in segs {
                if let StrSeg::Expr(e) = s {
                    f(e);
                }
            }
        }
        Expr::If(c, t, els, _) => {
            f(c);
            block_exprs(t, f);
            if let Some(e) = els {
                f(e);
            }
        }
        Expr::IfLet(_, s, t, els, _) => {
            f(s);
            block_exprs(t, f);
            if let Some(e) = els {
                f(e);
            }
        }
        Expr::Match(s, arms, _) => {
            f(s);
            for a in arms {
                f(&a.body);
            }
        }
        Expr::Block(b, _) => block_exprs(b, f),
        _ => {}
    }
}

fn block_exprs(b: &Block, f: &mut impl FnMut(&Expr)) {
    for s in &b.stmts {
        match s {
            Stmt::Let {
                value: Some(value), ..
            } => f(value),
            Stmt::Let { value: None, .. } => {}
            Stmt::Assign { target, value, .. } => {
                f(target);
                f(value);
            }
            Stmt::Expr(e) => f(e),
            Stmt::While { cond, body, .. } => {
                f(cond);
                block_exprs(body, f);
            }
            Stmt::For { iter, body, .. } => {
                f(iter);
                block_exprs(body, f);
            }
            Stmt::Return(Some(e), _) => f(e),
            _ => {}
        }
    }
}

/// Whether a body auto-wraps plain returns in `Result.Ok`, i.e. it uses
/// `error(...)` or the `expr!` propagation operator. Mirrors codegen's
/// `fallible_block` so MIR records the same fallibility bit.
pub fn fallible_block(b: &Block) -> bool {
    b.stmts.iter().any(fallible_stmt)
}

fn fallible_stmt(s: &Stmt) -> bool {
    match s {
        Stmt::Let { value, .. } => value.as_ref().is_some_and(fallible_expr),
        Stmt::Assign { target, value, .. } => fallible_expr(target) || fallible_expr(value),
        Stmt::Expr(e) => fallible_expr(e),
        Stmt::While { cond, body, .. } => fallible_expr(cond) || fallible_block(body),
        Stmt::For { iter, body, .. } => fallible_expr(iter) || fallible_block(body),
        Stmt::Return(Some(e), _) => fallible_expr(e),
        _ => false,
    }
}

fn fallible_expr(e: &Expr) -> bool {
    match e {
        Expr::ErrorProp(..) => true,
        Expr::Call(c, args, _) => {
            matches!(&**c, Expr::Ident(n, _) if n == "error")
                || fallible_expr(c)
                || args.iter().any(|a| fallible_expr(&a.expr))
        }
        Expr::Unary(_, a, _) | Expr::Field(a, _, _) => fallible_expr(a),
        Expr::Binary(_, a, b, _) | Expr::Index(a, b, _) => fallible_expr(a) || fallible_expr(b),
        Expr::Array(es, _) => es.iter().any(fallible_expr),
        Expr::TypeLit(_, fs, _) | Expr::VariantLit(_, _, fs, _) => {
            fs.iter().any(|(_, v)| fallible_expr(v))
        }
        Expr::Str(segs, _) => segs
            .iter()
            .any(|s| matches!(s, StrSeg::Expr(e) if fallible_expr(e))),
        Expr::If(c, t, e, _) => {
            fallible_expr(c) || fallible_block(t) || e.as_ref().is_some_and(|e| fallible_expr(e))
        }
        Expr::IfLet(_, s, t, e, _) => {
            fallible_expr(s) || fallible_block(t) || e.as_ref().is_some_and(|e| fallible_expr(e))
        }
        Expr::Match(s, arms, _) => fallible_expr(s) || arms.iter().any(|a| fallible_expr(&a.body)),
        Expr::Block(b, _) => fallible_block(b),
        _ => false,
    }
}
