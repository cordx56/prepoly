//! Convert the front end's `(message, span)` diagnostics into LSP diagnostics
//! ranged in the active document.

use brass_parser::Span;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity};

use crate::document::Document;

/// Map document-local `(message, span)` diagnostics to LSP `Diagnostic`s. Spans
/// are already document-local (see [`crate::analysis`]); only the line/column
/// mapping remains.
pub fn to_lsp(diags: &[(String, Span)], doc: &Document) -> Vec<Diagnostic> {
    diags
        .iter()
        .map(|(message, span)| Diagnostic {
            range: doc.range_of(*span),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("brass".to_string()),
            message: message.clone(),
            ..Default::default()
        })
        .collect()
}
