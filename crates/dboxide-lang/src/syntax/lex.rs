use crate::{
  syntax::green::{SyntaxToken, cache::Cache},
  syntax::ast::SyntaxKind,
  types::{PeekableStream, diagnostics::Diagnostic},
};

pub struct Lexer<'a> {
  stream: PeekableStream<'a, char>,
  offset: usize,
  cache: &'a mut Cache,
}

pub struct LexResult {
  pub token: SyntaxToken,
  pub diagnostic: Option<Diagnostic>,
}

impl<'a> Lexer<'a> {
  pub fn new(stream: PeekableStream<'a, char>, cache: &'a mut Cache) -> Self {
    Self {
      stream,
      offset: 0,
      cache,
    }
  }

  pub fn lex(&mut self) -> Option<LexResult> {
    None
  }

  fn token(&mut self, kind: SyntaxKind, text: &[u8]) -> SyntaxToken {
    SyntaxToken::new(self.cache, kind, text)
  }

  fn is_eof(&mut self) -> bool {
    self.stream.peek().is_none()
  }

  fn peek(&mut self) -> Option<char> {
    self.stream.peek().copied()
  }

  fn advance(&mut self) -> Option<char> {
    let ch = self.stream.next();
    if ch.is_some() {
      self.offset += 1;
    }
    ch
  }
}
