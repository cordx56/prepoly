//! Shared helpers for the position-driven features (hover, go-to-definition):
//! finding the identifier under the cursor, finding the tightest typed
//! expression at an offset, and turning a global span into an LSP `Location`.

use prepoly_hir::TypedExpr;
use prepoly_lexer::{Span, TokenKind, lex};
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
