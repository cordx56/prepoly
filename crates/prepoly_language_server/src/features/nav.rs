//! Shared helpers for the position-driven features (hover, go-to-definition):
//! finding the identifier under the cursor, finding the tightest typed
//! expression at an offset, and turning a global span into an LSP `Location`.

use prepoly_hir::{Type, TypedExpr, TypedExprKind};
use prepoly_lexer::{Span, TokenKind, lex};
use prepoly_parser::ast::{Block, Expr, Member, Module, Param, Pattern, Stmt, TopLevel, TypeBody};
use tower_lsp_server::ls_types::{Location, Uri};

use crate::analysis::FullAnalysis;
use crate::document::LineIndex;

/// The identifier token containing document-local offset `off`, as
/// `(name, local span)`. Used to know what symbol the cursor is on.
pub fn ident_at(text: &str, off: usize) -> Option<(String, Span)> {
    let toks = lex(text).ok()?;
    toks.into_iter().find_map(|t| match t.kind {
        TokenKind::Ident(name) if off >= t.span.lo && off <= t.span.hi => Some((name, t.span)),
        _ => None,
    })
}

/// The smallest typed expression whose global span contains `global_off`.
pub fn smallest_typed_at(full: &FullAnalysis, global_off: usize) -> Option<&TypedExpr> {
    full.typed
        .expressions
        .iter()
        .filter(|e| global_off >= e.span.lo && global_off <= e.span.hi)
        .min_by_key(|e| e.span.hi - e.span.lo)
}

/// Turn a global span into a `Location`, resolving the file it lives in through
/// the analysis source map. Returns `None` for a span in the embedded prelude
/// (it has no file to open).
pub fn locate(full: &FullAnalysis, span: Span) -> Option<Location> {
    let (path, src, lo_local) = full.sources.locate(span.lo)?;
    let path = path?;
    let hi_local = lo_local + span.hi.saturating_sub(span.lo);
    let index = LineIndex::new(src);
    let range = index.range_of(src, lo_local, hi_local);
    let uri = Uri::from_file_path(path)?;
    Some(Location { uri, range })
}

pub fn contains(span: Span, off: usize) -> bool {
    off >= span.lo && off <= span.hi
}

fn within(outer: Span, inner: Span) -> bool {
    inner.lo >= outer.lo && inner.hi <= outer.hi
}

/// The parameters and body of the function or method whose body contains
/// `global_off`.
pub fn enclosing(main_ast: &Module, global_off: usize) -> Option<(Vec<&Param>, &Block)> {
    for item in &main_ast.items {
        match item {
            TopLevel::Fun(f) if contains(f.body.span, global_off) => {
                return Some((f.params.iter().collect(), &f.body));
            }
            TopLevel::Type(t) => {
                let members = match &t.body {
                    TypeBody::Record(members) => members,
                    TypeBody::Sum(_) => continue,
                };
                for m in members {
                    if let Member::Method(method) = m
                        && let Some(body) = &method.body
                        && contains(body.span, global_off)
                    {
                        return Some((method.params.iter().collect(), body));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// The inferred type of the local variable `name` whose declaration or use is at
/// `global_off`. The type checker records expression nodes (variable *uses*) but
/// not binding sites, so a hover on a `let`, parameter, for-loop, or pattern
/// binding finds nothing under the cursor directly; this recovers the type from
/// the bound value, or from a use of the variable in the same function.
pub fn local_var_type(full: &FullAnalysis, global_off: usize, name: &str) -> Option<Type> {
    let (_, body) = enclosing(&full.main_ast, global_off)?;
    // Precise: the cursor is on a `let name = value` binding -> the value's type.
    if let Some(value_span) = let_value_span(body, global_off, name)
        && let Some(e) = full.typed.expressions.iter().find(|e| e.span == value_span)
    {
        return Some(e.ty.clone());
    }
    // Otherwise borrow the type from a use of the name in this function -- the
    // case for parameters, for-loop variables, and pattern bindings, which have
    // no bound value expression. Prefer the nearest use at or after the cursor,
    // so hovering a pattern binding picks the use in that same match arm rather
    // than a same-named binding in another arm.
    let mut uses: Vec<&TypedExpr> = full
        .typed
        .expressions
        .iter()
        .filter(|e| {
            matches!(&e.kind, TypedExprKind::Ident(n) if n == name) && within(body.span, e.span)
        })
        .collect();
    uses.sort_by_key(|e| e.span.lo);
    let after = uses.iter().find(|e| e.span.lo >= global_off);
    after.or_else(|| uses.last()).map(|e| e.ty.clone())
}

/// Span of the value expression of a `let name = value` whose binding identifier
/// contains `off`, searching nested blocks (loop/if/match/closure bodies).
fn let_value_span(block: &Block, off: usize, name: &str) -> Option<Span> {
    block
        .stmts
        .iter()
        .find_map(|s| let_value_in_stmt(s, off, name))
}

fn let_value_in_stmt(s: &Stmt, off: usize, name: &str) -> Option<Span> {
    match s {
        Stmt::Let { pat, value, .. } => {
            if let Pattern::Binding(n, bspan) = pat
                && n == name
                && contains(*bspan, off)
            {
                return Some(value.span());
            }
            let_value_in_expr(value, off, name)
        }
        Stmt::Assign { value, .. } => let_value_in_expr(value, off, name),
        Stmt::Expr(e) | Stmt::Return(Some(e), _) => let_value_in_expr(e, off, name),
        Stmt::While { body, .. } | Stmt::For { body, .. } => let_value_span(body, off, name),
        _ => None,
    }
}

fn let_value_in_expr(e: &Expr, off: usize, name: &str) -> Option<Span> {
    match e {
        Expr::Block(b, _) => let_value_span(b, off, name),
        Expr::If(_, then, els, _) | Expr::IfLet(_, _, then, els, _) => {
            let_value_span(then, off, name)
                .or_else(|| els.as_ref().and_then(|e| let_value_in_expr(e, off, name)))
        }
        Expr::Match(_, arms, _) => arms
            .iter()
            .find_map(|a| let_value_in_expr(&a.body, off, name)),
        Expr::Closure(_, body, _) => let_value_in_expr(body, off, name),
        _ => None,
    }
}
