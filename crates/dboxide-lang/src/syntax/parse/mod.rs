use std::{cell::RefCell, rc::Rc};

mod constants;
mod expr_ctx;
mod grammar;

#[cfg(test)]
mod tests;

use crate::{
  PeekableStream,
  ast::{GreenNode, SyntaxKind, cache::Cache},
  diagnostics::Diagnostic,
  lex::{LexCtx, LexResult},
  parse::{
    constants::{SKIP_COMMENT, SKIP_NEWLINE, SKIP_NONE, SKIP_WC, SKIP_WS},
    expr_ctx::ExprCtxStack,
  },
};

pub struct ParseCtx<'a> {
  pub(in crate::syntax::parse) cache: Rc<RefCell<Cache>>,
  pub(in crate::syntax::parse) tokens: PeekableStream<'a, LexResult>,
  pub(in crate::syntax::parse) diagnostics: Vec<Diagnostic>,
  pub(in crate::syntax::parse) expr_ctx_stack: ExprCtxStack,
  pub(in crate::syntax::parse) offset: usize,
}

pub struct ParseResult {
  pub ast: GreenNode,
  pub diagnostics: Vec<Diagnostic>,
}

impl<'a> ParseCtx<'a> {
  pub fn new(stream: PeekableStream<'a, char>, cache: Rc<RefCell<Cache>>) -> Self {
    let tokens = itertools::multipeek(
      Box::new(LexCtx::new(stream, cache.clone())) as Box<dyn Iterator<Item = LexResult>>
    );
    Self {
      cache,
      tokens,
      diagnostics: Vec::new(),
      expr_ctx_stack: ExprCtxStack::new(),
      offset: 0,
    }
  }

  pub fn parse(mut self) -> ParseResult {
    let root = self.source_file();
    ParseResult {
      ast: root,
      diagnostics: self.diagnostics,
    }
  }
}

/* Token stream primitives */
impl<'a> ParseCtx<'a> {
  /// Whether the given token kind should be skipped given the skip flags
  pub(in crate::syntax::parse) fn should_skip(&self, kind: SyntaxKind, skip: usize) -> bool {
    match kind {
      SyntaxKind::Whitespace => skip & SKIP_WS != 0,
      SyntaxKind::LineComment | SyntaxKind::BlockComment => skip & SKIP_COMMENT != 0,
      SyntaxKind::Newline => skip & SKIP_NEWLINE != 0,
      _ => false,
    }
  }

  /// Consume the next non-skipped token, pushing skipped trivia and result into children
  pub(in crate::syntax::parse) fn advance(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: usize,
  ) -> LexResult {
    self.tokens.reset_peek();
    loop {
      let Some(mut result) = self.tokens.next() else {
        unreachable!("Token stream is infinite");
      };

      if let Some(diag) = result.diagnostic.take() {
        self.diagnostics.push(diag);
      }
      self.offset = result.end_offset;
      children.push(GreenNode::from_token(result.token.clone()));

      if result.token.kind() == SyntaxKind::Eof || !self.should_skip(result.token.kind(), skip) {
        return result;
      }
    }
  }

  /// Like advance(), but expects the token to match `expected`
  pub(in crate::syntax::parse) fn consume(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: usize,
    expected: SyntaxKind,
    diagnostic: Diagnostic,
  ) -> bool {
    let result = self.advance(children, skip);
    if result.token.kind() != expected {
      let bad_token = children.pop().unwrap();
      children.push(self.emit(SyntaxKind::Error, &[bad_token]));
      self.diagnostics.push(diagnostic);
      false
    } else {
      true
    }
  }

  /// Synchronize: skip tokens until we find one that some context on the stack can handle
  /// Skipped tokens are wrapped in an Error node
  pub(in crate::syntax::parse) fn synchronize(&mut self, children: &mut Vec<GreenNode>) {
    let mut error_children = Vec::new();
    let peek = self.peek(SKIP_NONE);
    if peek.token.kind() != SyntaxKind::Eof {
      self.advance(&mut error_children, SKIP_NONE);
    }
    while self.peek(SKIP_NONE).token.kind() != SyntaxKind::Eof {
      let kind = self.peek(SKIP_NONE).token.kind();
      if self.expr_ctx_stack.find_handler(kind).is_some() {
        break;
      }
      self.advance(&mut error_children, SKIP_NONE);
    }
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
  }

  /// Consume trivia tokens (ws/comments/newlines) into children without consuming the next real token
  pub(in crate::syntax::parse) fn consume_trivia(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: usize,
  ) {
    self.tokens.reset_peek();
    loop {
      let Some(result) = self.tokens.peek().cloned() else {
        break;
      };
      if result.token.kind() == SyntaxKind::Eof || !self.should_skip(result.token.kind(), skip) {
        break;
      }
      self.tokens.reset_peek();

      let mut result = self.tokens.next().unwrap();
      if let Some(diag) = result.diagnostic.take() {
        self.diagnostics.push(diag);
      }
      self.offset = result.end_offset;
      children.push(GreenNode::from_token(result.token));
    }
    self.tokens.reset_peek();
  }

  pub(in crate::syntax::parse) fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.push(diagnostic);
  }

  pub(in crate::syntax::parse) fn peek(&mut self, skip: usize) -> LexResult {
    self.peek_nth(0, skip)
  }

  pub(in crate::syntax::parse) fn peeker(&mut self) -> Peeker<'a, '_> {
    self.tokens.reset_peek();
    Peeker { ctx: self }
  }

  pub(in crate::syntax::parse) fn peek_nth(&mut self, n: usize, skip: usize) -> LexResult {
    let mut peeker = self.peeker();
    for _ in 0..n {
      let tok = peeker.next_token(skip);
      if tok.token.kind() == SyntaxKind::Eof {
        return tok;
      }
    }
    peeker.next_token(skip)
  }

  pub(in crate::syntax::parse) fn expr_skip_flags(&self) -> usize {
    let mut skip = SKIP_WC;
    if self.expr_ctx_stack.should_expr_span_newline() {
      skip |= SKIP_NEWLINE;
    }
    skip
  }

  pub(in crate::syntax::parse) fn emit(
    &mut self,
    kind: SyntaxKind,
    children: &[GreenNode],
  ) -> GreenNode {
    debug_assert!(
      !kind.is_trivia(),
      "[ParseCtx::emit] Nodes cannot be trivia kinds"
    );
    GreenNode::from_node(self.cache.borrow_mut().node(kind, children))
  }
}

/// A stateful lookahead cursor over the token stream
/// Automatically resets peek state on Drop
pub(in crate::syntax::parse) struct Peeker<'a, 'b> {
  ctx: &'b mut ParseCtx<'a>,
}

impl<'a, 'b> Peeker<'a, 'b> {
  /// Advances the peek cursor and returns the next non-skipped token
  pub(in crate::syntax::parse) fn next_token(&mut self, skip: usize) -> LexResult {
    loop {
      let Some(result) = self.ctx.tokens.peek().cloned() else {
        unreachable!("Token stream is infinite");
      };
      let kind = result.token.kind();
      if kind == SyntaxKind::Eof || !self.ctx.should_skip(kind, skip) {
        return result;
      }
    }
  }
}

impl<'a, 'b> Drop for Peeker<'a, 'b> {
  fn drop(&mut self) {
    self.ctx.tokens.reset_peek();
  }
}
