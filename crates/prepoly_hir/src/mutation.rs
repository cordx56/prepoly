//! Parameter-mutation analysis over the collected program.
//!
//! A parameter (or a method's `self`) is *mutated* when the body writes through
//! the reference it names -- a field/element assignment (`p.f = ` / `p[i] = `),
//! a built-in growable-array mutator (`p.push(..)`), or by forwarding the
//! parameter into a callee position already known to mutate it. This is the
//! single signal that decides how an unannotated parameter is passed: a
//! non-mutated parameter is a shared borrow (`ref`), a mutated one is a private
//! deep copy (`mut`), and a mutated `self` is a mutable reference (`ref(mut)`).
//!
//! The analysis is a pure function of the HIR program (bodies are still parser
//! AST) and is shared by const-checking, the back ends' copy machinery, and the
//! language server's type display so all three agree on which arguments are
//! borrowed and which are copied.

use std::collections::{HashMap, HashSet};

use prepoly_parser::ast::{Block, Expr, Stmt, TypeExpr};

use crate::{FunInfo, ParamInfo, Program, Type, TypeKind};

/// Precomputed write-through facts for every callable in a program: which
/// parameter positions a `const` value must not be passed into because the
/// callee could mutate it *through the reference* and so change the caller's
/// value. Only a `ref(mut(T))` parameter writes through; an unannotated mutated
/// parameter is a private deep copy (`mut`), so mutating it is const-safe and it
/// is not recorded here.
pub struct MutationInfo {
    /// Free-function storage symbol -> indices of parameters that write through
    /// to the caller: a `ref(mut(T))` parameter, or one that forwards a place
    /// rooted at it into another write-through position. A least fixpoint over
    /// the call graph.
    functions: HashMap<String, HashSet<usize>>,
    /// `(type name, method name)` pairs whose body mutates `self` *through a
    /// write-through `self`* (an unannotated `self`, inferred `ref(mut(Self))`,
    /// or an explicit `ref(mut(Self))`). A method taking `self: Self`/`mut(Self)`
    /// mutates only its own copy, so it is excluded.
    self_write_through_methods: HashSet<(String, String)>,
}

impl MutationInfo {
    /// Analyze every free function and method in `program`.
    pub fn analyze(program: &Program) -> Self {
        MutationInfo {
            functions: write_through_function_params(program),
            self_write_through_methods: self_write_through_methods(program),
        }
    }

    /// The indices of free function `symbol`'s write-through parameters, if any.
    /// Passing a `const` value into one of these is rejected.
    pub fn write_through_params(&self, symbol: &str) -> Option<&HashSet<usize>> {
        self.functions.get(symbol)
    }

    /// Whether method `type_name::method` mutates the caller's receiver through a
    /// write-through `self`, so calling it on a `const` receiver is rejected.
    pub fn method_writes_through_self(&self, type_name: &str, method: &str) -> bool {
        self.self_write_through_methods
            .contains(&(type_name.to_string(), method.to_string()))
    }
}

/// Whether a `self` parameter writes through to the caller's receiver: an
/// unannotated `self` (a reference by default) or an explicit `ref(mut(Self))`.
/// An immutable `ref(Self)` cannot mutate at all, and `self: Self`/`mut(Self)`
/// mutates only an owned copy -- neither reaches the caller.
fn self_writes_through(self_param: &ParamInfo) -> bool {
    match &self_param.ty {
        None => true,
        Some(TypeExpr::Ref(inner, _)) => matches!(**inner, TypeExpr::Mut(..)),
        Some(_) => false,
    }
}

/// The `(type name, method name)` pairs whose body mutates a write-through `self`.
fn self_write_through_methods(program: &Program) -> HashSet<(String, String)> {
    let writes_through = |method: &crate::MethodInfo| {
        method
            .signature
            .params
            .first()
            .is_some_and(self_writes_through)
            && method
                .decl
                .body
                .as_ref()
                .is_some_and(|body| mutates_root(body, "self"))
    };
    program
        .types
        .values()
        .flat_map(|info| match &info.kind {
            TypeKind::Record { methods, .. } => methods
                .iter()
                .filter(|(_, method)| writes_through(method))
                .map(|(name, _)| (info.name.clone(), name.clone()))
                .collect::<Vec<_>>(),
            TypeKind::Sum { variants } => variants
                .iter()
                .flat_map(|variant| {
                    variant
                        .methods
                        .iter()
                        .filter(|(_, method)| writes_through(method))
                        .map(|(name, _)| (info.name.clone(), name.clone()))
                })
                .collect::<Vec<_>>(),
        })
        .collect()
}

/// Parameter indices each function writes through to its caller, keyed by
/// storage symbol. A position writes through when the parameter is annotated
/// `ref(mut(T))`, or the body forwards a place rooted at it into another
/// write-through position. A `const` argument must not be passed into these.
///
/// Only `ref(mut(T))` writes through: a directly-mutated unannotated (or `mut`,
/// or bare-aggregate) parameter is a private deep copy, so mutating it never
/// reaches the caller and it is *not* recorded -- passing a `const` into it is
/// safe. The forwarding case makes this interprocedural: the table is a least
/// fixpoint over the call graph. Each round, a parameter `p` of `f` is added if
/// `f`'s body passes a place rooted at `p` into a callee position already known
/// to write through. Iteration repeats until no set grows; the sets only gain
/// elements (monotone) and are bounded by the parameter count, so it terminates.
///
/// Scope boundary: only free functions carry per-parameter entries here; a
/// method that forwards a non-`self` parameter into a write-through call is not
/// covered.
fn write_through_function_params(program: &Program) -> HashMap<String, HashSet<usize>> {
    // Seed with the directly write-through parameters (`ref(mut(T))`).
    let mut map: HashMap<String, HashSet<usize>> = HashMap::new();
    for f in program.functions.values() {
        let indices: HashSet<usize> = f
            .signature
            .params
            .iter()
            .enumerate()
            .filter(|(_, p)| param_is_mut_ref(p))
            .map(|(i, _)| i)
            .collect();
        if !indices.is_empty() {
            map.insert(f.symbol.clone(), indices);
        }
    }
    // Propagate through forwarding calls until the fixpoint is reached.
    loop {
        let mut changed = false;
        for f in program.functions.values() {
            for (param_idx, p) in f.signature.params.iter().enumerate() {
                if param_is_copied(p) {
                    // A deep-copied parameter's mutation never reaches the caller,
                    // so forwarding it does not make this position mutable.
                    continue;
                }
                if map.get(&f.symbol).is_some_and(|s| s.contains(&param_idx)) {
                    continue;
                }
                if forwards_param_to_mutating(program, f, &p.name, &map) {
                    map.entry(f.symbol.clone()).or_default().insert(param_idx);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    map
}

/// Whether `f`'s body passes a place rooted at parameter `root` into a position
/// some callee already requires to be mutable (per the in-progress `mutating`
/// table). This is the interprocedural step of [`mutating_function_params`]:
/// `fun f(p) { g(p) }` where `g` mutates its parameter makes `p` mutable too.
fn forwards_param_to_mutating(
    program: &Program,
    f: &FunInfo,
    root: &str,
    mutating: &HashMap<String, HashSet<usize>>,
) -> bool {
    let mut found = false;
    forwards_in_block(program, &f.module, &f.decl.body, root, mutating, &mut found);
    found
}

fn forwards_in_block(
    program: &Program,
    module: &[String],
    block: &Block,
    root: &str,
    mutating: &HashMap<String, HashSet<usize>>,
    found: &mut bool,
) {
    for stmt in &block.stmts {
        if *found {
            return;
        }
        match stmt {
            Stmt::Let { value, .. } => {
                forwards_in_expr(program, module, value, root, mutating, found)
            }
            Stmt::Assign { target, value, .. } => {
                forwards_in_expr(program, module, target, root, mutating, found);
                forwards_in_expr(program, module, value, root, mutating, found);
            }
            Stmt::While { cond, body, .. } => {
                forwards_in_expr(program, module, cond, root, mutating, found);
                forwards_in_block(program, module, body, root, mutating, found);
            }
            Stmt::For { iter, body, .. } => {
                forwards_in_expr(program, module, iter, root, mutating, found);
                forwards_in_block(program, module, body, root, mutating, found);
            }
            Stmt::Expr(e) | Stmt::Return(Some(e), _) => {
                forwards_in_expr(program, module, e, root, mutating, found)
            }
            Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }
}

fn forwards_in_expr(
    program: &Program,
    module: &[String],
    expr: &Expr,
    root: &str,
    mutating: &HashMap<String, HashSet<usize>>,
    found: &mut bool,
) {
    if *found {
        return;
    }
    match expr {
        // A free-function call `g(.., arg_i, ..)`: if `g`'s position `i` is
        // mutable and `arg_i` is a place rooted at `root`, the parameter escapes
        // into a mutating position.
        Expr::Call(callee, args, _) => {
            if let Expr::Ident(fname, _) = callee.as_ref()
                && let Some(symbol) = program.resolve_fn_symbol(module, fname)
                && let Some(indices) = mutating.get(&symbol)
            {
                for (i, arg) in args.iter().enumerate() {
                    if indices.contains(&i) && root_ident(&arg.expr) == Some(root) {
                        *found = true;
                        return;
                    }
                }
            }
            forwards_in_expr(program, module, callee, root, mutating, found);
            for arg in args {
                forwards_in_expr(program, module, &arg.expr, root, mutating, found);
            }
        }
        Expr::Unary(_, inner, _) | Expr::ErrorProp(inner, _) | Expr::Field(inner, _, _) => {
            forwards_in_expr(program, module, inner, root, mutating, found)
        }
        Expr::Binary(_, left, right, _) | Expr::Range(left, right, _) => {
            forwards_in_expr(program, module, left, root, mutating, found);
            forwards_in_expr(program, module, right, root, mutating, found);
        }
        Expr::Index(base, idx, _) => {
            forwards_in_expr(program, module, base, root, mutating, found);
            forwards_in_expr(program, module, idx, root, mutating, found);
        }
        Expr::Closure(_, body, _) => forwards_in_expr(program, module, body, root, mutating, found),
        Expr::Array(items, _) => {
            for item in items {
                forwards_in_expr(program, module, item, root, mutating, found);
            }
        }
        Expr::TypeLit(_, fields, _) | Expr::VariantLit(_, _, fields, _) => {
            for (_, value) in fields {
                forwards_in_expr(program, module, value, root, mutating, found);
            }
        }
        Expr::If(cond, then, els, _) => {
            forwards_in_expr(program, module, cond, root, mutating, found);
            forwards_in_block(program, module, then, root, mutating, found);
            if let Some(els) = els {
                forwards_in_expr(program, module, els, root, mutating, found);
            }
        }
        Expr::IfLet(_, scrut, then, els, _) => {
            forwards_in_expr(program, module, scrut, root, mutating, found);
            forwards_in_block(program, module, then, root, mutating, found);
            if let Some(els) = els {
                forwards_in_expr(program, module, els, root, mutating, found);
            }
        }
        Expr::Match(scrut, arms, _) => {
            forwards_in_expr(program, module, scrut, root, mutating, found);
            for arm in arms {
                forwards_in_expr(program, module, &arm.body, root, mutating, found);
            }
        }
        Expr::Block(block, _) => forwards_in_block(program, module, block, root, mutating, found),
        Expr::Int(..)
        | Expr::Float(..)
        | Expr::Str(..)
        | Expr::Bool(..)
        | Expr::Null(_)
        | Expr::Ident(..)
        | Expr::SelfExpr(_) => {}
    }
}

/// Whether a parameter is a mutable reference (`ref(mut(T))`).
pub fn param_is_mut_ref(p: &ParamInfo) -> bool {
    matches!(&p.resolved_ty, Some(Type::Ref(inner)) if matches!(**inner, Type::Mut(_)))
}

/// Whether a parameter is passed by deep copy: a non-reference array/slice (a
/// `mut(...)` wrapper does not change that). Such a parameter's mutations are
/// confined to the callee's copy, so a `const` argument to it is fine.
pub fn param_is_copied(p: &ParamInfo) -> bool {
    fn peel(t: &Type) -> &Type {
        match t {
            Type::Mut(inner) => peel(inner),
            _ => t,
        }
    }
    matches!(&p.resolved_ty, Some(t)
        if !matches!(t, Type::Ref(_)) && matches!(peel(t), Type::Slice(_) | Type::Array(..)))
}

/// Whether a parameter is an immutable reference (`ref(T)`, not `ref(mut(T))`).
pub fn param_is_immutable_ref(p: &ParamInfo) -> bool {
    matches!(&p.resolved_ty, Some(Type::Ref(inner)) if !matches!(**inner, Type::Mut(_)))
}

/// The built-in growable-array mutators: a method that mutates its receiver in
/// place rather than producing a fresh value. They make their receiver mutable.
fn is_builtin_mutating_method(method: &str) -> bool {
    matches!(method, "push" | "insert" | "remove" | "pop")
}

/// Whether `block` mutates the value behind `root` *through the reference* it
/// names -- a field/element assignment (`root.f = ` / `root[i] = `) or a built-in
/// mutating method (`root.push(..)`), including through nested projections. This
/// is the signal that makes a parameter (or `self`) mutable: such a mutation is
/// visible to the caller. A bare `root = ...` only rebinds the local and is *not*
/// counted -- it does not touch the caller's value, so a `const` argument bound to
/// a copied or rebindable parameter stays valid.
pub fn mutates_root(block: &Block, root: &str) -> bool {
    block.stmts.iter().any(|stmt| stmt_mutates_root(stmt, root))
}

fn stmt_mutates_root(stmt: &Stmt, root: &str) -> bool {
    match stmt {
        Stmt::Assign { target, .. } => {
            matches!(target, Expr::Field(..) | Expr::Index(..)) && root_ident(target) == Some(root)
        }
        Stmt::While { body, .. } => mutates_root(body, root),
        // `for e in <place rooted at root> { .. }` binds each element by reference
        // of the array's kind, so writing back through the loop variable `e`
        // (`e = ..`, `e.f = ..`, `e.push(..)`) mutates `root`'s elements. A direct
        // mutation may also appear in the body.
        Stmt::For {
            var, iter, body, ..
        } => {
            (root_ident(iter) == Some(root) && writes_binding(body, var))
                || mutates_root(body, root)
        }
        Stmt::Expr(expr) => expr_mutates_root(expr, root),
        Stmt::Return(Some(expr), _) => expr_mutates_root(expr, root),
        Stmt::Let { value, .. } => expr_mutates_root(value, root),
        Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => false,
    }
}

/// Whether `block` writes through the binding `name`: a reassignment
/// (`name = ..`, `name *= ..`), a field/element store (`name.f = ..`,
/// `name[i] = ..`), or a built-in mutating method (`name.push(..)`). Unlike
/// [`mutates_root`] this counts a bare reassignment, because for a loop variable
/// a reassignment writes the element back into the iterated array. A nested `for`
/// that rebinds the same name shadows it and is not counted.
fn writes_binding(block: &Block, name: &str) -> bool {
    block.stmts.iter().any(|stmt| match stmt {
        Stmt::Assign { target, .. } => root_ident(target) == Some(name),
        Stmt::For {
            var, iter, body, ..
        } => expr_writes_binding(iter, name) || (var != name && writes_binding(body, name)),
        Stmt::While { cond, body, .. } => {
            expr_writes_binding(cond, name) || writes_binding(body, name)
        }
        Stmt::Expr(e) | Stmt::Return(Some(e), _) | Stmt::Let { value: e, .. } => {
            expr_writes_binding(e, name)
        }
        Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => false,
    })
}

fn expr_writes_binding(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Block(block, _) => writes_binding(block, name),
        Expr::If(_, then, els, _) | Expr::IfLet(_, _, then, els, _) => {
            writes_binding(then, name)
                || els
                    .as_ref()
                    .is_some_and(|els| expr_writes_binding(els, name))
        }
        Expr::Match(_, arms, _) => arms.iter().any(|arm| expr_writes_binding(&arm.body, name)),
        Expr::Closure(_, body, _) => expr_writes_binding(body, name),
        Expr::Call(callee, args, _) => {
            matches!(&**callee, Expr::Field(recv, m, _)
                if is_builtin_mutating_method(m) && root_ident(recv) == Some(name))
                || args.iter().any(|a| expr_writes_binding(&a.expr, name))
        }
        _ => false,
    }
}

fn expr_mutates_root(expr: &Expr, root: &str) -> bool {
    match expr {
        Expr::Block(block, _) => mutates_root(block, root),
        Expr::If(_, then, els, _) | Expr::IfLet(_, _, then, els, _) => {
            mutates_root(then, root) || els.as_ref().is_some_and(|els| expr_mutates_root(els, root))
        }
        Expr::Match(_, arms, _) => arms.iter().any(|arm| expr_mutates_root(&arm.body, root)),
        Expr::Closure(_, body, _) => expr_mutates_root(body, root),
        // A built-in array mutator on a place rooted at `root` mutates it; a
        // mutation may also appear inside an argument expression.
        Expr::Call(callee, args, _) => {
            matches!(&**callee, Expr::Field(recv, m, _)
                if is_builtin_mutating_method(m) && root_ident(recv) == Some(root))
                || args.iter().any(|a| expr_mutates_root(&a.expr, root))
        }
        _ => false,
    }
}

/// The base identifier a place expression is rooted at (`a.b[c]` -> `a`).
pub fn root_ident(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Ident(name, _) => Some(name),
        Expr::SelfExpr(_) => Some("self"),
        Expr::Field(base, _, _) | Expr::Index(base, _, _) => root_ident(base),
        _ => None,
    }
}

/// Whether a parameter's passing mode is inferred from whether the body mutates
/// it, rather than fixed by an annotation. Only a truly unannotated parameter
/// qualifies: it becomes a shared borrow (`ref`) when not mutated and a private
/// deep copy (`mut`) when mutated. An explicit `a: infer` is *not* inferred -- it
/// is always a read-only deep copy -- and every other annotation states its own
/// passing mode via its `ref`/`mut` wrapper.
pub fn param_infers_pass_mode(p: &ParamInfo) -> bool {
    p.ty.is_none()
}

/// Whether a parameter is an explicit `infer` (`a: infer`): a read-only deep
/// copy whose contents the body may not mutate.
pub fn param_is_infer(p: &ParamInfo) -> bool {
    matches!(&p.ty, Some(TypeExpr::Named(n, _)) if n == "infer")
}
