//! The Prepoly lexer.
//!
//! Converts source text into a token stream. Newlines become explicit
//! `Newline` tokens (a run of blank lines collapses to one); the parser
//! decides whether each one terminates a statement.

use crate::token::{Span, StrPart, Token, TokenKind, keyword_or_ident};

/// A lexing error with the offending source span.
#[derive(Clone, Debug, PartialEq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    tokens: Vec<Token>,
}

/// Tokenize `src` into a token vector terminated by `Eof`.
///
/// Leading newlines and newlines adjacent to other newlines are collapsed so
/// the parser never sees empty statements. Comments are discarded but a
/// newline that ends a `//` comment still separates statements.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let mut lx = Lexer {
        src: src.as_bytes(),
        pos: 0,
        tokens: Vec::new(),
    };
    lx.run()?;
    Ok(lx.tokens)
}

impl<'a> Lexer<'a> {
    fn run(&mut self) -> Result<(), LexError> {
        loop {
            // Skip non-newline whitespace and comments, emitting Newline tokens.
            self.skip_trivia()?;
            if self.pos >= self.src.len() {
                break;
            }
            let c = self.src[self.pos];
            match c {
                b'0'..=b'9' => self.lex_number()?,
                b'"' => self.lex_string()?,
                c if is_ident_start(c) => self.lex_ident(),
                _ => self.lex_symbol()?,
            }
        }
        let end = self.src.len();
        self.tokens
            .push(Token::new(TokenKind::Eof, Span::new(end, end)));
        Ok(())
    }

    /// Skip spaces, tabs, carriage returns, and comments. Newlines emit a
    /// single `Newline` token (collapsing runs and suppressing a leading one).
    fn skip_trivia(&mut self) -> Result<(), LexError> {
        loop {
            if self.pos >= self.src.len() {
                return Ok(());
            }
            let c = self.src[self.pos];
            match c {
                b' ' | b'\t' | b'\r' => self.pos += 1,
                b'\n' => {
                    self.pos += 1;
                    self.push_newline();
                }
                b'/' if self.peek(1) == Some(b'/') => {
                    while self.pos < self.src.len() && self.src[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                }
                b'/' if self.peek(1) == Some(b'*') => self.skip_block_comment()?,
                _ => return Ok(()),
            }
        }
    }

    /// Skip a `/* ... */` block comment. Block comments nest.
    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        self.pos += 2;
        let mut depth = 1;
        while self.pos < self.src.len() && depth > 0 {
            if self.src[self.pos] == b'/' && self.peek(1) == Some(b'*') {
                depth += 1;
                self.pos += 2;
            } else if self.src[self.pos] == b'*' && self.peek(1) == Some(b'/') {
                depth -= 1;
                self.pos += 2;
            } else {
                // A newline inside a block comment does not separate statements.
                self.pos += 1;
            }
        }
        if depth != 0 {
            return Err(LexError {
                message: "unterminated block comment".into(),
                span: Span::new(start, self.pos),
            });
        }
        Ok(())
    }

    /// Emit a Newline token unless the previous token was already a Newline or
    /// this is the very first token.
    fn push_newline(&mut self) {
        match self.tokens.last() {
            None => {} // suppress leading newlines
            Some(t) if matches!(t.kind, TokenKind::Newline) => {}
            _ => self.tokens.push(Token::new(
                TokenKind::Newline,
                Span::new(self.pos - 1, self.pos),
            )),
        }
    }

    fn lex_ident(&mut self) {
        let start = self.pos;
        while self.pos < self.src.len() && is_ident_continue(self.src[self.pos]) {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.src[start..self.pos]).unwrap();
        let kind = keyword_or_ident(text);
        self.tokens
            .push(Token::new(kind, Span::new(start, self.pos)));
    }

    fn lex_number(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        // Radix prefixes: 0x, 0b, 0o.
        if self.src[self.pos] == b'0'
            && let Some(p) = self.peek(1)
        {
            let radix = match p {
                b'x' | b'X' => Some(16),
                b'b' | b'B' => Some(2),
                b'o' | b'O' => Some(8),
                _ => None,
            };
            if let Some(radix) = radix {
                self.pos += 2;
                let digits_start = self.pos;
                while self.pos < self.src.len()
                    && (self.src[self.pos] == b'_' || is_radix_digit(self.src[self.pos], radix))
                {
                    self.pos += 1;
                }
                let digits: String = self.src[digits_start..self.pos]
                    .iter()
                    .filter(|&&b| b != b'_')
                    .map(|&b| b as char)
                    .collect();
                if digits.is_empty() {
                    return Err(self.err(start, "missing digits after radix prefix"));
                }
                let value = i64::from_str_radix(&digits, radix)
                    .map_err(|_| self.err(start, "integer literal out of range"))?;
                self.tokens.push(Token::new(
                    TokenKind::Int(value),
                    Span::new(start, self.pos),
                ));
                return Ok(());
            }
        }

        // Decimal integer part.
        while self.pos < self.src.len()
            && (self.src[self.pos].is_ascii_digit() || self.src[self.pos] == b'_')
        {
            self.pos += 1;
        }

        let mut is_float = false;
        // Fractional part: only when '.' is followed by a digit (so `42.foo`
        // stays an int followed by field access).
        if self.cur() == Some(b'.') && self.peek(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            is_float = true;
            self.pos += 1;
            while self.pos < self.src.len()
                && (self.src[self.pos].is_ascii_digit() || self.src[self.pos] == b'_')
            {
                self.pos += 1;
            }
        }
        // Exponent part.
        if matches!(self.cur(), Some(b'e') | Some(b'E')) {
            let mut look = self.pos + 1;
            if matches!(self.src.get(look), Some(b'+') | Some(b'-')) {
                look += 1;
            }
            if self
                .src
                .get(look)
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                is_float = true;
                self.pos = look;
                while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
            }
        }

        let text: String = self.src[start..self.pos]
            .iter()
            .filter(|&&b| b != b'_')
            .map(|&b| b as char)
            .collect();
        let kind = if is_float {
            TokenKind::Float(
                text.parse()
                    .map_err(|_| self.err(start, "invalid float literal"))?,
            )
        } else {
            TokenKind::Int(
                text.parse()
                    .map_err(|_| self.err(start, "integer literal out of range"))?,
            )
        };
        self.tokens
            .push(Token::new(kind, Span::new(start, self.pos)));
        Ok(())
    }

    /// Lex a string literal with `{expr}` interpolation and escapes.
    fn lex_string(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        self.pos += 1; // opening quote
        let mut parts: Vec<StrPart> = Vec::new();
        let mut cur = String::new();
        loop {
            if self.pos >= self.src.len() {
                return Err(self.err(start, "unterminated string literal"));
            }
            let c = self.src[self.pos];
            match c {
                b'"' => {
                    self.pos += 1;
                    break;
                }
                b'\\' => {
                    self.pos += 1;
                    let e = self
                        .cur()
                        .ok_or_else(|| self.err(start, "unterminated escape"))?;
                    let resolved = match e {
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        b'0' => '\0',
                        b'\\' => '\\',
                        b'"' => '"',
                        b'{' => '{',
                        b'}' => '}',
                        _ => return Err(self.err(self.pos, "unknown escape sequence")),
                    };
                    cur.push(resolved);
                    self.pos += 1;
                }
                b'{' => {
                    if !cur.is_empty() {
                        parts.push(StrPart::Lit(std::mem::take(&mut cur)));
                    }
                    let expr = self.read_interpolation(start)?;
                    parts.push(StrPart::Interp(expr));
                }
                _ => {
                    // Copy raw UTF-8 byte; multi-byte chars pass through intact.
                    cur.push(c as char);
                    // Fix up multi-byte: if not ASCII, decode the full char.
                    if c >= 0x80 {
                        cur.pop();
                        let s = std::str::from_utf8(&self.src[self.pos..])
                            .map_err(|_| self.err(self.pos, "invalid utf-8"))?;
                        let ch = s.chars().next().unwrap();
                        cur.push(ch);
                        self.pos += ch.len_utf8();
                        continue;
                    }
                    self.pos += 1;
                }
            }
        }
        if !cur.is_empty() || parts.is_empty() {
            parts.push(StrPart::Lit(cur));
        }
        self.tokens.push(Token::new(
            TokenKind::Str(parts),
            Span::new(start, self.pos),
        ));
        Ok(())
    }

    /// Read the raw source of an interpolated `{expr}`, balancing braces and
    /// skipping over nested string literals so their braces are not counted.
    fn read_interpolation(&mut self, str_start: usize) -> Result<String, LexError> {
        self.pos += 1; // consume '{'
        let expr_start = self.pos;
        let mut depth = 1;
        while self.pos < self.src.len() {
            let c = self.src[self.pos];
            match c {
                b'{' => {
                    depth += 1;
                    self.pos += 1;
                }
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let raw = std::str::from_utf8(&self.src[expr_start..self.pos])
                            .map_err(|_| self.err(self.pos, "invalid utf-8"))?
                            .to_string();
                        self.pos += 1; // consume '}'
                        return Ok(raw);
                    }
                    self.pos += 1;
                }
                b'"' => self.skip_nested_string()?,
                _ => self.pos += 1,
            }
        }
        Err(self.err(str_start, "unterminated interpolation"))
    }

    /// Skip a nested string literal while inside an interpolation, honoring
    /// escapes so a `\"` does not end it prematurely.
    fn skip_nested_string(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        self.pos += 1;
        while self.pos < self.src.len() {
            match self.src[self.pos] {
                b'\\' => self.pos += 2,
                b'"' => {
                    self.pos += 1;
                    return Ok(());
                }
                _ => self.pos += 1,
            }
        }
        Err(self.err(start, "unterminated nested string"))
    }

    fn lex_symbol(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        let c = self.src[self.pos];
        let two = self.peek(1);
        // Two-character operators first.
        let (kind, len) = match (c, two) {
            (b'=', Some(b'>')) => (TokenKind::FatArrow, 2),
            (b'=', Some(b'=')) => (TokenKind::EqEq, 2),
            (b'!', Some(b'=')) => (TokenKind::NotEq, 2),
            (b'<', Some(b'=')) => (TokenKind::LtEq, 2),
            (b'>', Some(b'=')) => (TokenKind::GtEq, 2),
            (b'&', Some(b'&')) => (TokenKind::AmpAmp, 2),
            (b'|', Some(b'|')) => (TokenKind::PipePipe, 2),
            (b'<', Some(b'<')) => (TokenKind::Shl, 2),
            (b'>', Some(b'>')) => (TokenKind::Shr, 2),
            (b'+', Some(b'=')) => (TokenKind::PlusEq, 2),
            (b'-', Some(b'=')) => (TokenKind::MinusEq, 2),
            (b'*', Some(b'=')) => (TokenKind::StarEq, 2),
            (b'/', Some(b'=')) => (TokenKind::SlashEq, 2),
            (b'%', Some(b'=')) => (TokenKind::PercentEq, 2),
            (b'-', Some(b'>')) => (TokenKind::Arrow, 2),
            (b'+', _) => (TokenKind::Plus, 1),
            (b'-', _) => (TokenKind::Minus, 1),
            (b'*', _) => (TokenKind::Star, 1),
            (b'/', _) => (TokenKind::Slash, 1),
            (b'%', _) => (TokenKind::Percent, 1),
            (b'=', _) => (TokenKind::Eq, 1),
            (b'<', _) => (TokenKind::Lt, 1),
            (b'>', _) => (TokenKind::Gt, 1),
            (b'!', _) => (TokenKind::Bang, 1),
            (b'&', _) => (TokenKind::Amp, 1),
            (b'|', _) => (TokenKind::Pipe, 1),
            (b'^', _) => (TokenKind::Caret, 1),
            (b'~', _) => (TokenKind::Tilde, 1),
            (b'?', _) => (TokenKind::Question, 1),
            (b'.', _) => (TokenKind::Dot, 1),
            (b',', _) => (TokenKind::Comma, 1),
            (b':', _) => (TokenKind::Colon, 1),
            (b'(', _) => (TokenKind::LParen, 1),
            (b')', _) => (TokenKind::RParen, 1),
            (b'[', _) => (TokenKind::LBracket, 1),
            (b']', _) => (TokenKind::RBracket, 1),
            (b'{', _) => (TokenKind::LBrace, 1),
            (b'}', _) => (TokenKind::RBrace, 1),
            _ => return Err(self.err(start, "unexpected character")),
        };
        self.pos += len;
        self.tokens
            .push(Token::new(kind, Span::new(start, self.pos)));
        Ok(())
    }

    fn cur(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn peek(&self, n: usize) -> Option<u8> {
        self.src.get(self.pos + n).copied()
    }

    fn err(&self, lo: usize, msg: &str) -> LexError {
        LexError {
            message: msg.to_string(),
            span: Span::new(lo, self.pos.max(lo + 1)),
        }
    }
}

fn is_ident_start(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphabetic()
}

fn is_ident_continue(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphanumeric()
}

fn is_radix_digit(c: u8, radix: u32) -> bool {
    (c as char).is_digit(radix)
}
