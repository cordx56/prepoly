//! In-memory document model: source text plus a line index that converts
//! between the byte-offset spans the compiler produces and the line/column
//! positions the LSP protocol speaks.
//!
//! LSP `Position` columns are UTF-16 code units (the protocol default), which
//! differ from both byte offsets and `char` counts, so the conversions here own
//! that encoding rather than reusing `prepoly_lexer::line_col` (which counts
//! Unicode scalar values for human-facing diagnostics).

use prepoly_lexer::Span;
use tower_lsp_server::ls_types::{Position, Range};

/// A source document and a precomputed index of its line boundaries.
pub struct Document {
    pub text: String,
    /// Byte offset at which each 0-based line begins; always starts with `0`.
    line_starts: Vec<usize>,
    /// Client-reported version, bumped on every change. Used to tag published
    /// diagnostics and to invalidate the cached full analysis.
    pub version: i32,
}

impl Document {
    pub fn new(text: String, version: i32) -> Self {
        let line_starts = line_starts(&text);
        Document {
            text,
            line_starts,
            version,
        }
    }

    pub fn update(&mut self, text: String, version: i32) {
        self.line_starts = line_starts(&text);
        self.text = text;
        self.version = version;
    }

    /// Byte offset -> LSP position. The column is the number of UTF-16 code
    /// units between the line start and `offset`.
    pub fn position_at(&self, offset: usize) -> Position {
        let offset = offset.min(self.text.len());
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(next) => next - 1,
        };
        let line_start = self.line_starts[line];
        let character = self.text[line_start..offset]
            .chars()
            .map(|c| c.len_utf16() as u32)
            .sum();
        Position {
            line: line as u32,
            character,
        }
    }

    /// LSP position -> byte offset. Out-of-range lines and columns clamp to the
    /// nearest valid offset so a stale position never panics.
    pub fn offset_at(&self, pos: Position) -> usize {
        let line = pos.line as usize;
        let Some(&line_start) = self.line_starts.get(line) else {
            return self.text.len();
        };
        let line_end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.text.len());
        let mut remaining = pos.character;
        let mut offset = line_start;
        for c in self.text[line_start..line_end].chars() {
            let w = c.len_utf16() as u32;
            if w > remaining {
                break;
            }
            remaining -= w;
            offset += c.len_utf8();
        }
        offset
    }

    pub fn range_of(&self, span: Span) -> Range {
        Range {
            start: self.position_at(span.lo),
            end: self.position_at(span.hi),
        }
    }
}

/// A standalone line index for an arbitrary source string, used to map spans in
/// files other than the active document (e.g. a go-to-definition target in an
/// imported file) without constructing a full `Document`.
pub struct LineIndex {
    line_starts: Vec<usize>,
    len: usize,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        LineIndex {
            line_starts: line_starts(text),
            len: text.len(),
        }
    }

    pub fn position_at(&self, text: &str, offset: usize) -> Position {
        let offset = offset.min(self.len);
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(next) => next - 1,
        };
        let line_start = self.line_starts[line];
        let character = text[line_start..offset]
            .chars()
            .map(|c| c.len_utf16() as u32)
            .sum();
        Position {
            line: line as u32,
            character,
        }
    }

    pub fn range_of(&self, text: &str, lo: usize, hi: usize) -> Range {
        Range {
            start: self.position_at(text, lo),
            end: self.position_at(text, hi),
        }
    }
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}
