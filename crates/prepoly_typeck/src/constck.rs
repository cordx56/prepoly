//! Const checking (DESIGN.md 5.4): a value bound with `const` cannot be
//! reassigned, nor can its fields/elements be mutated. Assignments whose root
//! place is a const binding are rejected. Calls to methods inferred to mutate
//! `self` are also rejected when their receiver is a const binding with a known
//! record type.

use std::collections::{HashMap, HashSet};

use prepoly_hir::{Program, Type, TypeKind};
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
    mutating_methods: HashSet<(String, String)>,
    /// Function name -> indices of parameters the body directly reassigns a
    /// field/element of. Used to reject passing a const value into a position
    /// that the callee mutates (DESIGN.md 5.4 / 5.6 interprocedural rule). This
    /// is a conservative approximation: it covers direct `param.field = ...`
    /// mutation, not transitive mutation through further calls.
    mutating_params: HashMap<String, HashSet<usize>>,
    errors: Vec<TypeError>,
}

pub fn check(program: &Program) -> Vec<TypeError> {
    let mut checker = ConstChecker {
        program,
        mutating_methods: mutating_methods(program),
        mutating_params: mutating_function_params(program),
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
            self.check_block(&f.decl.body, &mut vec![globals.clone()]);
        }
        for t in self.program.types.values() {
            let bodies: Vec<&Block> = match &t.kind {
                TypeKind::Record { methods, .. } => methods
                    .values()
                    .filter_map(|m| m.decl.body.as_ref())
                    .collect(),
                TypeKind::Sum { variants } => variants
                    .iter()
                    .flat_map(|v| v.methods.values().filter_map(|m| m.decl.body.as_ref()))
                    .collect(),
            };
            for body in bodies {
                self.check_block(body, &mut vec![globals.clone()]);
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

    /// The top-level `const` bindings visible to every body in the file
    /// (DESIGN.md 2.5, 5.4). A later top-level `let` that reuses a name shadows
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
                            // the same shared value, so constness propagates
                            // (DESIGN.md 5.4). The runtime shares heap objects by
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
            Stmt::For { iter, body, .. } => {
                self.check_expr(iter, scopes);
                self.check_block(body, scopes);
            }
            Stmt::Expr(expr) => self.check_expr(expr, scopes),
            Stmt::Return(Some(expr), _) => self.check_expr(expr, scopes),
            Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr, scopes: &mut ConstScopes) {
        match expr {
            Expr::Unary(_, inner, _) | Expr::ErrorProp(inner, _) => self.check_expr(inner, scopes),
            Expr::Binary(_, left, right, _) => {
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
        // The receiver may be a nested projection of a const value
        // (e.g. `o.inner.bump()`); a mutating method on any field reachable from
        // a const root is rejected (DESIGN.md 5.4 const propagation).
        let Some(type_name) = self.const_place_type(receiver, scopes) else {
            return;
        };
        if self
            .mutating_methods
            .contains(&(type_name, method.to_string()))
        {
            let root = root_ident(receiver).unwrap_or("");
            self.errors.push(TypeError {
                message: format!("cannot call mutating method `{method}` on const value `{root}`"),
                span,
            });
        }
    }

    /// Reject passing a const record/sum value into a parameter position that
    /// the callee mutates. The callee taking ownership of the mutation would let
    /// it modify a value declared immutable at the call site.
    fn check_const_args_to_mutating_fn(
        &mut self,
        callee: &Expr,
        args: &[Arg],
        scopes: &ConstScopes,
    ) {
        let Expr::Ident(fname, _) = callee else {
            return;
        };
        let Some(indices) = self.mutating_params.get(fname).cloned() else {
            return;
        };
        for (i, arg) in args.iter().enumerate() {
            if indices.contains(&i) && self.const_place_type(&arg.expr, scopes).is_some() {
                let root = root_ident(&arg.expr).unwrap_or("");
                self.errors.push(TypeError {
                    message: format!(
                        "cannot pass const value `{root}` to `{fname}`, which mutates that parameter"
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

fn mutating_methods(program: &Program) -> HashSet<(String, String)> {
    program
        .types
        .values()
        .flat_map(|info| match &info.kind {
            TypeKind::Record { methods, .. } => methods
                .iter()
                .filter_map(|(name, method)| {
                    method.decl.body.as_ref().and_then(|body| {
                        mutates_root(body, "self").then(|| (info.name.clone(), name.clone()))
                    })
                })
                .collect::<Vec<_>>(),
            TypeKind::Sum { variants } => variants
                .iter()
                .flat_map(|variant| {
                    variant.methods.iter().filter_map(|(name, method)| {
                        method.decl.body.as_ref().and_then(|body| {
                            mutates_root(body, "self").then(|| (info.name.clone(), name.clone()))
                        })
                    })
                })
                .collect::<Vec<_>>(),
        })
        .collect()
}

/// Parameter indices each function directly mutates (assigns a field/element of
/// a place rooted at the parameter). Conservative: transitive mutation through
/// nested calls is not tracked.
fn mutating_function_params(program: &Program) -> HashMap<String, HashSet<usize>> {
    let mut map = HashMap::new();
    for f in program.functions.values() {
        let indices: HashSet<usize> = f
            .signature
            .params
            .iter()
            .enumerate()
            .filter(|(_, p)| mutates_root(&f.decl.body, &p.name))
            .map(|(i, _)| i)
            .collect();
        if !indices.is_empty() {
            map.insert(f.signature.name.clone(), indices);
        }
    }
    map
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

fn mutates_root(block: &Block, root: &str) -> bool {
    block.stmts.iter().any(|stmt| stmt_mutates_root(stmt, root))
}

fn stmt_mutates_root(stmt: &Stmt, root: &str) -> bool {
    match stmt {
        Stmt::Assign { target, .. } => root_ident(target) == Some(root),
        Stmt::While { body, .. } | Stmt::For { body, .. } => mutates_root(body, root),
        Stmt::Expr(expr) => expr_mutates_root(expr, root),
        Stmt::Return(Some(expr), _) => expr_mutates_root(expr, root),
        Stmt::Let { value, .. } => expr_mutates_root(value, root),
        Stmt::Return(None, _) | Stmt::Break(_) | Stmt::Continue(_) => false,
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
        _ => false,
    }
}

/// The base identifier a place expression is rooted at (`a.b[c]` -> `a`).
fn root_ident(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Ident(name, _) => Some(name),
        Expr::SelfExpr(_) => Some("self"),
        Expr::Field(base, _, _) | Expr::Index(base, _, _) => root_ident(base),
        _ => None,
    }
}
