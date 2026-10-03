#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::rc::Rc;

use crate::syntax::ast::SyntaxKind;
use crate::syntax::ast::{SyntaxToken, cache::Cache};
use crate::types::PeekableStream;
use crate::types::diagnostics::{Diagnostic, DiagnosticKind};

pub struct LexCtx<'a> {
  stream: PeekableStream<'a, char>,
  offset: usize,
  buffered_text: Vec<u8>,
  cache: Rc<RefCell<Cache>>,
}

// Infinite stream, ends with infinite Eof
impl<'a> Iterator for LexCtx<'a> {
  type Item = LexResult;

  fn next(&mut self) -> Option<Self::Item> {
    Some(self.lex())
  }
}

#[derive(Clone)]
pub struct LexResult {
  pub token: SyntaxToken,
  pub diagnostic: Option<Diagnostic>,
  pub start_offset: usize,
  pub end_offset: usize,
}

impl<'a> LexCtx<'a> {
  pub fn new(stream: PeekableStream<'a, char>, cache: Rc<RefCell<Cache>>) -> Self {
    Self {
      stream,
      offset: 0,
      buffered_text: Vec::new(),
      cache,
    }
  }

  pub fn lex(&mut self) -> LexResult {
    self.buffered_text.clear();
    let char = match self.peek() {
      Some(char) => char,
      None => {
        return self.emit(SyntaxKind::Eof);
      }
    };

    match char {
      '\r' => {
        self.advance();
        if self.peek().is_some_and(|char| char == '\n') {
          self.advance();
        }
        self.emit(SyntaxKind::Newline)
      }
      '\n' => {
        self.advance();
        self.emit(SyntaxKind::Newline)
      }
      _ if char.is_ascii_whitespace() => self.whitespace(),
      ':' => {
        self.advance();
        self.emit(SyntaxKind::Colon)
      }
      ',' => {
        self.advance();
        self.emit(SyntaxKind::Comma)
      }
      '(' => {
        self.advance();
        self.emit(SyntaxKind::LParen)
      }
      ')' => {
        self.advance();
        self.emit(SyntaxKind::RParen)
      }
      '[' => {
        self.advance();
        self.emit(SyntaxKind::LBracket)
      }
      ']' => {
        self.advance();
        self.emit(SyntaxKind::RBracket)
      }
      '{' => {
        self.advance();
        self.emit(SyntaxKind::LBrace)
      }
      '}' => {
        self.advance();
        self.emit(SyntaxKind::RBrace)
      }
      '"' => self.dq_string(),
      '\'' => self.sq_string(),
      '`' => self.oq_string(),
      _ if char.is_ascii_digit() => self.number(),
      _ if char == '_' || char.is_ascii_alphabetic() => self.ident(),
      '#' => self.color_lit(),
      _ if is_operator_char(char) => self.operator(),
      _ => {
        let offset = self.offset;
        self.advance();
        let diagnostic = DiagnosticKind::UnexpectedChar { ch: char }.into_error(offset, offset + 1);
        self.emit_with(SyntaxKind::Error, Some(diagnostic))
      }
    }
  }

  fn whitespace(&mut self) -> LexResult {
    while let Some(char) = self.peek() {
      if char == '\n' || !char.is_ascii_whitespace() {
        break;
      }
      self.advance();
    }
    self.emit(SyntaxKind::Whitespace)
  }

  fn ident(&mut self) -> LexResult {
    while let Some(char) = self.peek() {
      if !char.is_ascii_alphanumeric() && char != '_' {
        break;
      }
      self.advance();
    }
    self.emit(SyntaxKind::Ident)
  }

  fn number(&mut self) -> LexResult {
    while let Some(char) = self.peek() {
      if !char.is_ascii_digit() {
        break;
      }
      self.advance();
    }
    // fractional: peek 2 chars ahead (`.` then digit)
    if self.peek() == Some('.') && self.peek_nth(1).is_some_and(|char| char.is_ascii_digit()) {
      self.advance(); // '.'
      while let Some(char) = self.peek() {
        if !char.is_ascii_digit() {
          break;
        }
        self.advance();
      }
    }
    // exponent
    if matches!(self.peek(), Some('e' | 'E')) {
      self.advance();
      if matches!(self.peek(), Some('+' | '-')) {
        self.advance();
      }
      while let Some(char) = self.peek() {
        if !char.is_ascii_digit() {
          break;
        }
        self.advance();
      }
    }
    self.emit(SyntaxKind::Number)
  }

  fn color_lit(&mut self) -> LexResult {
    let start = self.offset;
    self.advance(); // consume '#'
    let mut count = 0usize;
    while let Some(char) = self.peek() {
      if !char.is_ascii_hexdigit() {
        break;
      }
      self.advance();
      count += 1;
    }
    if count == 0 {
      let diagnostic = DiagnosticKind::UnexpectedChar { ch: '#' }.into_error(start, start + 1);
      return self.emit_with(SyntaxKind::Error, Some(diagnostic));
    }
    if count != 3 && count != 6 {
      let end = self.offset;
      let diagnostic = DiagnosticKind::InvalidColorLitLength { count }.into_error(start, end);
      return self.emit_with(SyntaxKind::Error, Some(diagnostic));
    }
    self.emit(SyntaxKind::ColorLit)
  }

  fn operator(&mut self) -> LexResult {
    let start = self.offset;
    let first = self.peek().unwrap();
    self.advance();

    if first == '/' {
      if self.peek() == Some('/') {
        self.advance();
        while let Some(char) = self.peek() {
          if char == '\n' {
            break;
          }
          self.advance();
        }
        return self.emit(SyntaxKind::LineComment);
      }
      if self.peek() == Some('*') {
        self.advance();
        loop {
          if self.at_end() {
            let diagnostic =
              DiagnosticKind::UnexpectedEof { expected: '/' }.into_error(start, self.offset);
            return self.emit_with(SyntaxKind::Error, Some(diagnostic));
          }
          if self.peek() == Some('*') && self.peek_nth(1) == Some('/') {
            self.advance(); // *
            self.advance(); // /
            return self.emit(SyntaxKind::BlockComment);
          }
          self.advance();
        }
      }
    }

    while let Some(char) = self.peek() {
      if is_operator_char(char) || char == ':' {
        self.advance();
      } else {
        break;
      }
    }
    self.emit(SyntaxKind::Operator)
  }

  fn dq_string(&mut self) -> LexResult {
    let start = self.offset;
    self.advance(); // "
    loop {
      match self.peek() {
        None | Some('\n') => {
          let diagnostic = DiagnosticKind::UnterminatedString.into_error(start, self.offset);
          return self.emit_with(SyntaxKind::Error, Some(diagnostic));
        }
        Some('\\') => {
          self.advance();
          self.advance();
        }
        Some('"') => {
          self.advance();
          return self.emit(SyntaxKind::DqString);
        }
        _ => {
          self.advance();
        }
      }
    }
  }

  fn sq_string(&mut self) -> LexResult {
    let start = self.offset;
    self.advance(); // '
    if self.peek() == Some('\'') {
      self.advance();
      if self.peek() == Some('\'') {
        self.advance();
        loop {
          if self.at_end() {
            let diagnostic = DiagnosticKind::UnterminatedString.into_error(start, self.offset);
            return self.emit_with(SyntaxKind::Error, Some(diagnostic));
          }
          if self.peek() == Some('\'')
            && self.peek_nth(1) == Some('\'')
            && self.peek_nth(2) == Some('\'')
          {
            self.advance();
            self.advance();
            self.advance();
            return self.emit(SyntaxKind::TqString);
          }
          self.advance();
        }
      }
      return self.emit(SyntaxKind::SqString);
    }
    loop {
      match self.peek() {
        None | Some('\n') => {
          let diagnostic = DiagnosticKind::UnterminatedString.into_error(start, self.offset);
          return self.emit_with(SyntaxKind::Error, Some(diagnostic));
        }
        Some('\\') => {
          self.advance();
          self.advance();
        }
        Some('\'') => {
          self.advance();
          return self.emit(SyntaxKind::SqString);
        }
        _ => {
          self.advance();
        }
      }
    }
  }

  fn oq_string(&mut self) -> LexResult {
    let start = self.offset;
    self.advance(); // `
    loop {
      match self.peek() {
        None => {
          let diagnostic = DiagnosticKind::UnterminatedString.into_error(start, self.offset);
          return self.emit_with(SyntaxKind::Error, Some(diagnostic));
        }
        Some('\\') if self.peek_nth(1) == Some('`') => {
          self.advance();
          self.advance();
        }
        Some('`') => {
          self.advance();
          return self.emit(SyntaxKind::OqString);
        }
        _ => {
          self.advance();
        }
      }
    }
  }
}

/// Returns true if the character can appear in an operator token
fn is_operator_char(char: char) -> bool {
  matches!(
    char,
    '.' | '@' | '|' | '?' | '=' | '<' | '>' | '!' | '-' | '+' | '*' | '/' | '%' | '^' | '&' | '~'
  )
}

impl<'a> LexCtx<'a> {
  fn advance(&mut self) {
    if let Some(char) = self.stream.next() {
      let mut bytes = [0u8; 4];
      let encoded = char.encode_utf8(&mut bytes);
      self.buffered_text.extend_from_slice(encoded.as_bytes());
      self.offset += encoded.len();
    }
  }

  fn peek(&mut self) -> Option<char> {
    self.stream.reset_peek();
    self.stream.peek().copied()
  }

  fn peek_nth(&mut self, n: usize) -> Option<char> {
    self.stream.reset_peek();
    let mut result = None;
    for _ in 0..=n {
      result = self.stream.peek().copied();
    }
    self.stream.reset_peek();
    result
  }

  fn at_end(&mut self) -> bool {
    self.stream.peek().is_none()
  }

  fn emit(&mut self, kind: SyntaxKind) -> LexResult {
    self.emit_with(kind, None)
  }

  fn emit_with(&mut self, kind: SyntaxKind, diagnostic: Option<Diagnostic>) -> LexResult {
    debug_assert!(
      self.offset >= self.buffered_text.len(),
      "[LexCtx::emit] Offset invariant violated"
    );
    let token = SyntaxToken::new(self.cache.clone(), kind, &self.buffered_text);
    LexResult {
      token,
      diagnostic,
      end_offset: self.offset,
      start_offset: self.offset - self.buffered_text.len(),
    }
  }
}
