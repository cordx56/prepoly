//! Const checking: a value bound with `const` cannot be
//! reassigned, nor can its fields/elements be mutated. Assignments whose root
//! place is a const binding are rejected. Calls to methods inferred to mutate
//! `self` are also rejected when their receiver is a const binding with a known
//! record type.

use std::collections::HashMap;

use prepoly_hir::{
    MutationInfo, ParamInfo, Program, Type, TypeKind, mutates_root, param_is_immutable_ref,
    param_is_infer, root_ident,
};
use prepoly_parser::ast::*;

use crate::TypeError;

/// A binding tracked for const checking. `Const` carries the receiver type name
/// when it is statically known, which is used to detect mutating-method calls.
/// `Mutable` records a non-const `let` so that it shadows an outer const of the
/// same name (e.g. a local that reuses a global const's name), suppressing
/// false positives on assignment to the inner binding.
#[derive(Clone)]
enum Binding {
    Const(Option<String>),
    Mutable,
}

type ConstScopes = Vec<HashMap<String, Binding>>;

struct ConstChecker<'a> {
    program: &'a Program,
    /// Shared parameter-mutation facts: which free-function parameters and which
    /// methods' `self` the bodies mutate. Const checking uses this to reject
    /// passing a const value into a position the callee mutates through its
    /// reference (a `ref(mut(..))` parameter, an unannotated mutated parameter,
    /// or a mutating method on a const receiver).
    mutation: MutationInfo,
    /// The module whose body is currently being checked, so a bare callee name at
    /// a call site resolves to the same symbol the fixpoint keyed on.
    current_module: Vec<String>,
    errors: Vec<TypeError>,
}

pub fn check(program: &Program) -> Vec<TypeError> {
    let mut checker = ConstChecker {
        program,
        mutation: MutationInfo::analyze(program),
        current_module: Vec::new(),
        errors: Vec::new(),
    };
    checker.check_program();
    checker.errors
}

impl ConstChecker<'_> {
    fn check_program(&mut self) {
        // Top-level consts are in scope for every body in the file, so function
        // and method bodies start from the module's global const bindings.
        let globals = self.global_consts();
        for f in self.program.functions.values() {
            // Parameters are mutable handles within their own body (mutating one is
            // the very thing that makes the parameter mutable to callers), and they
            // shadow any like-named global const. A `ref(T)` parameter is an
            // immutable reference, so it binds as const instead.
            let params = param_scope(&f.signature.params, false);
            self.current_module = f.module.clone();
            self.check_param_mutability(&f.signature.params, &f.decl.body);
            self.check_block(&f.decl.body, &mut vec![globals.clone(), params]);
        }
        for t in self.program.types.values() {
            let methods: Vec<&prepoly_hir::MethodInfo> = match &t.kind {
                TypeKind::Record { methods, .. } => methods.values().collect(),
                TypeKind::Sum { variants } => {
                    variants.iter().flat_map(|v| v.methods.values()).collect()
                }
            };
            for m in methods {
                let Some(body) = m.decl.body.as_ref() else {
                    continue;
                };
                // A method also binds `self` (mutable within the body). A method
                // body resolves callee names against the type's home module.
                let params = param_scope(&m.signature.params, true);
                self.current_module = t.module.clone();
                self.check_param_mutability(&m.signature.params, body);
                self.check_block(body, &mut vec![globals.clone(), params]);
            }
        }
        // Top-level init statements build their scope up in order, so a global
        // is only visible to later top-level statements, not earlier ones.
        for init in &self.program.inits {
            let mut scopes = vec![HashMap::new()];
            for stmt in &init.stmts {
                self.check_stmt(stmt, &mut scopes);
            }
        }
    }

    /// The top-level `const` bindings visible to every body in the file.
    /// A later top-level `let` that reuses a name shadows
    /// an earlier const, matching the order-sensitive init scope.
    fn global_consts(&self) -> HashMap<String, Binding> {
        let mut consts = HashMap::new();
        for init in &self.program.inits {
            for stmt in &init.stmts {
                let Stmt::Let {
                    pat: Pattern::Binding(name, _),
                    ty,
                    value,
                    is_const,
                    ..
                } = stmt
                else {
                    continue;
                };
                if *is_const {
                    consts.insert(
                        name.clone(),
                        Binding::Const(binding_type_name(self.program, ty, value)),
                    );
                } else {
                    consts.remove(name);
                }
            }
        }
        consts
    }

    fn check_block(&mut self, block: &Block, scopes: &mut ConstScopes) {
        scopes.push(HashMap::new());
        for stmt in &block.stmts {
            self.check_stmt(stmt, scopes);
        }
        scopes.pop();
    }

    fn check_stmt(&mut self, stmt: &Stmt, scopes: &mut ConstScopes) {
        match stmt {
            Stmt::Let {
                pat,
                ty,
                value,
                is_const,
                ..
            } => {
                self.check_expr(value, scopes);
                if let Pattern::Binding(name, _) = pat {
                    let alias = self.const_record_alias(value, scopes);
                    if let Some(top) = scopes.last_mut() {
                        let binding = if *is_const {
                            Binding::Const(binding_type_name(self.program, ty, value))
                        } else if let Some(type_name) = alias {
                            // Aliasing a const record/sum binds another handle to
                            // the same shared value, so constness propagates. The runtime shares heap objects by
                            // reference, hence the alias is also immutable.
                            Binding::Const(type_name)
                        } else {
                            // Record the shadow so it hides an outer const.
                            Binding::Mutable
                        };
                        top.insert(name.clone(), binding);
                    }
                }
            }
            Stmt::Assign { target, span, .. } => {
                self.check_expr(target, scopes);
                if let Some(root) = root_ident(target)
                    && matches!(self.const_binding(scopes, root), Some(Binding::Const(_)))
                {
                    self.errors.push(TypeError {
                        message: format!("cannot assign to const value `{root}`"),
                        span: *span,
                    });
                }
            }
            Stmt::While { cond, body, .. } => {
                self.check_expr(cond, scopes);
                self.check_block(body, scopes);
            }
            Stmt::For {
                var, iter, body, ..
            } => {
                self.check_expr(iter, scopes);
                // The loop variable is a reference into each element, so its
                // mutability matches the iterand: iterating an immutable array (a
                // const, or a `ref(T)` parameter) binds it const, rejecting an
                // in-place mutation like `e *= 2`. A mutable array binds it mutable.
                let binding = if self.const_root(iter, scopes).is_some() {
                    Binding::Const(None)
                } else {
                    Binding::Mutable
                };
                scopes.push(HashMap::from([(var.clone(), binding)]));
                self.check_block(body, scopes);
                scopes.pop();
            }
            Stmt::Expr(expr) => self.check_expr(expr, scopes),
            Stmt::Return(Some(expr), _) => self.check_expr(expr, scopes),
            Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr, scopes: &mut ConstScopes) {
        match expr {
            Expr::Unary(_, inner, _) | Expr::ErrorProp(inner, _) => self.check_expr(inner, scopes),
            Expr::Binary(_, left, right, _) | Expr::Range(left, right, _) => {
                self.check_expr(left, scopes);
                self.check_expr(right, scopes);
            }
            Expr::Call(callee, args, span) => {
                self.check_mutating_const_call(callee, *span, scopes);
                self.check_const_args_to_mutating_fn(callee, args, scopes);
                self.check_expr(callee, scopes);
                for arg in args {
                    self.check_expr(&arg.expr, scopes);
                }
            }
            Expr::Field(base, _, _) => self.check_expr(base, scopes),
            Expr::Index(base, idx, _) => {
                self.check_expr(base, scopes);
                self.check_expr(idx, scopes);
            }
            Expr::Closure(_, body, _) => self.check_expr(body, scopes),
            Expr::Array(items, _) => {
                for item in items {
                    self.check_expr(item, scopes);
                }
            }
            Expr::TypeLit(_, fields, _) | Expr::VariantLit(_, _, fields, _) => {
                for (_, value) in fields {
                    self.check_expr(value, scopes);
                }
            }
            Expr::If(cond, then, els, _) => {
                self.check_expr(cond, scopes);
                self.check_block(then, scopes);
                if let Some(els) = els {
                    self.check_expr(els, scopes);
                }
            }
            Expr::IfLet(_, scrutinee, then, els, _) => {
                self.check_expr(scrutinee, scopes);
                self.check_block(then, scopes);
                if let Some(els) = els {
                    self.check_expr(els, scopes);
                }
            }
            Expr::Match(scrutinee, arms, _) => {
                self.check_expr(scrutinee, scopes);
                for arm in arms {
                    self.check_expr(&arm.body, scopes);
                }
            }
            Expr::Block(block, _) => self.check_block(block, scopes),
            Expr::Int(..)
            | Expr::Float(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::Null(_)
            | Expr::Ident(..)
            | Expr::SelfExpr(_) => {}
        }
    }

    fn check_mutating_const_call(
        &mut self,
        callee: &Expr,
        span: prepoly_lexer::Span,
        scopes: &ConstScopes,
    ) {
        let Expr::Field(receiver, method, _) = callee else {
            return;
        };
        // A built-in growable-array mutator (`push`/`insert`/`remove`/`pop`) on a
        // const array -- or an array reachable from a const struct/sum/tuple root --
        // modifies a value declared immutable, so it is rejected. The receiver is a
        // const place that is not a user nominal type (records/sums whose own
        // methods are checked via `mutating_methods` below have a type name).
        if matches!(method.as_str(), "push" | "insert" | "remove" | "pop")
            && self.const_place_type(receiver, scopes).is_none()
            && let Some(root) = self.const_root(receiver, scopes)
        {
            self.errors.push(TypeError {
                message: format!("cannot call mutating method `{method}` on const value `{root}`"),
                span,
            });
            return;
        }
        // The receiver may be a nested projection of a const value
        // (e.g. `o.inner.bump()`); a mutating method on any field reachable from
        // a const root is rejected (propagation).
        let Some(type_name) = self.const_place_type(receiver, scopes) else {
            return;
        };
        if self.mutation.method_writes_through_self(&type_name, method) {
            let root = root_ident(receiver).unwrap_or("");
            self.errors.push(TypeError {
                message: format!("cannot call mutating method `{method}` on const value `{root}`"),
                span,
            });
        }
    }

    /// The const binding a place is rooted in (its root identifier), or `None`
    /// when the root is not const. Unlike `const_place_type` this does not require
    /// the place to be a user nominal type, so it also covers const arrays/tuples.
    fn const_root<'a>(&self, place: &'a Expr, scopes: &ConstScopes) -> Option<&'a str> {
        let root = root_ident(place)?;
        match self.const_binding(scopes, root) {
            Some(Binding::Const(_)) => Some(root),
            _ => None,
        }
    }

    /// Reject mutating a parameter whose type forbids it. An `a: infer` parameter
    /// receives a read-only deep copy, so mutating it through its reference is
    /// rejected here. The other read-only case -- an immutable `ref(T)` parameter
    /// (or `self: ref(Self)`) -- binds as const in [`param_scope`], so it is
    /// reported by the const path and skipped here to avoid a duplicate error.
    /// Everything else (an unannotated parameter, a `mut(T)`/`ref(mut(T))`, a bare
    /// aggregate value, or a mutable `self`) may be mutated.
    fn check_param_mutability(&mut self, params: &[ParamInfo], body: &Block) {
        for p in params {
            if param_permits_mutation(p) || param_is_immutable_ref(p) {
                continue;
            }
            if !mutates_root(body, &p.name) {
                continue;
            }
            self.errors.push(TypeError {
                message: format!(
                    "cannot mutate parameter `{}`: an `infer` parameter is a read-only deep copy \
                     (use `mut(T)` for a mutable copy, or `ref(mut(T))` for a mutable reference)",
                    p.name
                ),
                span: p.span,
            });
        }
    }

    /// Reject passing an immutable (`const`-rooted) value into a parameter the
    /// callee requires to be mutable: it mutates that parameter through the shared
    /// reference, which would modify a value declared immutable at the call site.
    /// Any const-rooted place qualifies (records, sums, arrays, tuples) -- a
    /// primitive parameter is never marked mutable (it cannot be mutated through a
    /// reference), so this does not reject a copied `const` primitive argument.
    fn check_const_args_to_mutating_fn(
        &mut self,
        callee: &Expr,
        args: &[Arg],
        scopes: &ConstScopes,
    ) {
        let Expr::Ident(fname, _) = callee else {
            return;
        };
        // Resolve the bare callee name to the same storage symbol the fixpoint
        // keyed on, as seen from the body's own module, so cross-module same-named
        // functions do not alias.
        let Some(symbol) = self.program.resolve_fn_symbol(&self.current_module, fname) else {
            return;
        };
        let Some(indices) = self.mutation.write_through_params(&symbol).cloned() else {
            return;
        };
        for (i, arg) in args.iter().enumerate() {
            if indices.contains(&i)
                && let Some(root) = self.const_root(&arg.expr, scopes)
            {
                self.errors.push(TypeError {
                    message: format!(
                        "cannot pass const value `{root}` to `{fname}`, which requires a mutable parameter"
                    ),
                    span: arg.expr.span(),
                });
            }
        }
    }

    /// The user type name of a place expression that is rooted in a const
    /// binding, following field projections through the type definitions. Returns
    /// `None` when the place is not const-rooted or its type is not a known
    /// nominal type.
    fn const_place_type(&self, place: &Expr, scopes: &ConstScopes) -> Option<String> {
        match place {
            Expr::Ident(name, _) => match self.const_binding(scopes, name) {
                Some(Binding::Const(ty)) => ty.clone(),
                _ => None,
            },
            Expr::SelfExpr(_) => match self.const_binding(scopes, "self") {
                Some(Binding::Const(ty)) => ty.clone(),
                _ => None,
            },
            Expr::Field(base, field, _) => {
                let base_ty = self.const_place_type(base, scopes)?;
                self.field_type_name(&base_ty, field)
            }
            _ => None,
        }
    }

    /// The nominal type name of a record field, if it has one.
    fn field_type_name(&self, type_name: &str, field: &str) -> Option<String> {
        let info = self.program.types.get(type_name)?;
        let TypeKind::Record { fields, .. } = &info.kind else {
            return None;
        };
        let resolved = fields.iter().find(|f| f.name == field)?.resolved_ty.clone();
        match resolved? {
            Type::Record(n) | Type::Sum(n) => Some(n.name().to_string()),
            _ => None,
        }
    }

    fn const_binding<'a>(&self, scopes: &'a ConstScopes, name: &str) -> Option<&'a Binding> {
        scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    /// The user type name of a const record/sum that `value` directly aliases,
    /// if any. Only a bare identifier (or `self`) bound to a const heap value
    /// qualifies: a method/function call produces a fresh value, and primitives
    /// are copied rather than shared, so neither propagates constness. Returning
    /// the type name lets mutating-method detection apply to the alias too.
    fn const_record_alias(&self, value: &Expr, scopes: &ConstScopes) -> Option<Option<String>> {
        let name = match value {
            Expr::Ident(name, _) => name.as_str(),
            Expr::SelfExpr(_) => "self",
            _ => return None,
        };
        match self.const_binding(scopes, name) {
            // A type name marks a heap record/sum, which is shared by reference;
            // a const without one is a primitive and is copied on binding.
            Some(Binding::Const(Some(type_name))) => Some(Some(type_name.clone())),
            _ => None,
        }
    }
}

/// A scope binding each parameter (and `self` for a method) so it shadows a
/// like-named global const. A `ref(T)` parameter is an immutable reference, so it
/// binds as const (mutating through it is rejected); every other parameter binds
/// as a mutable local (it owns its copy, or is a `ref(mut(T))` mutable reference).
fn param_scope(params: &[prepoly_hir::ParamInfo], is_method: bool) -> HashMap<String, Binding> {
    let mut scope = HashMap::new();
    if is_method {
        scope.insert("self".to_string(), Binding::Mutable);
    }
    for p in params {
        let binding = if param_is_immutable_ref(p) {
            Binding::Const(None)
        } else {
            Binding::Mutable
        };
        scope.insert(p.name.clone(), binding);
    }
    scope
}

/// Whether a parameter may be mutated through its reference in the body. Two
/// annotations forbid it: `a: infer` receives a read-only deep copy, and
/// `a: ref(T)`/`self: ref(Self)` is an immutable reference. Everything else -- an
/// unannotated parameter (a private `mut` copy, or an inferred `ref(mut(Self))`
/// for `self`), a `mut(T)`/`ref(mut(T))`, `self: Self`/`mut(Self)`, or a bare
/// aggregate value -- may be mutated. Immutable references are reported by the
/// const path (they bind as const), so callers also skip them to avoid a
/// duplicate error.
fn param_permits_mutation(p: &ParamInfo) -> bool {
    !param_is_infer(p) && !param_is_immutable_ref(p)
}

fn binding_type_name(program: &Program, ty: &Option<TypeExpr>, value: &Expr) -> Option<String> {
    match ty {
        Some(TypeExpr::Named(name, _)) if program.types.contains_key(name) => Some(name.clone()),
        _ => constructed_type_name(value).filter(|name| program.types.contains_key(name)),
    }
}

fn constructed_type_name(value: &Expr) -> Option<String> {
    match value {
        Expr::TypeLit(name, _, _) | Expr::VariantLit(name, _, _, _) => Some(name.clone()),
        _ => None,
    }
}
