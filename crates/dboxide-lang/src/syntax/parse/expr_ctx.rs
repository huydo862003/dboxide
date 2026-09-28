use crate::syntax::ast::SyntaxKind;

/* Expression context */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExprCtx {
  File,
  Block,
  Field,
  List,
  Tuple,
}

impl ExprCtx {
  /// Whether expressions in this context can span across newlines
  pub(in crate::syntax::parse) fn should_expr_span_newline(self) -> bool {
    matches!(self, ExprCtx::List | ExprCtx::Tuple)
  }

  /// Whether this context can handle (i.e. should stop synchronizing at) the given token kind
  pub(in crate::syntax::parse) fn can_handle(self, kind: SyntaxKind) -> bool {
    matches!(
      (self, kind),
      (ExprCtx::File, SyntaxKind::Eof)
        | (ExprCtx::File, SyntaxKind::Newline)
        | (ExprCtx::Block, SyntaxKind::RBrace)
        | (ExprCtx::Block, SyntaxKind::Newline)
        | (ExprCtx::Block, SyntaxKind::Eof)
        | (ExprCtx::Field, SyntaxKind::Newline)
        | (ExprCtx::Field, SyntaxKind::Eof)
        | (ExprCtx::List, SyntaxKind::RBracket)
        | (ExprCtx::List, SyntaxKind::Comma)
        | (ExprCtx::List, SyntaxKind::Eof)
        | (ExprCtx::Tuple, SyntaxKind::RParen)
        | (ExprCtx::Tuple, SyntaxKind::Comma)
        | (ExprCtx::Tuple, SyntaxKind::Eof)
    )
  }
}

/* Expression context stack */

pub(in crate::syntax::parse) struct ExprCtxStack {
  stack: Vec<ExprCtx>,
}

impl ExprCtxStack {
  pub(in crate::syntax::parse) fn new() -> Self {
    Self { stack: Vec::new() }
  }

  pub(in crate::syntax::parse) fn enter(&mut self, ctx: ExprCtx) {
    self.stack.push(ctx);
  }

  pub(in crate::syntax::parse) fn exit(&mut self, expected: ExprCtx) {
    debug_assert_eq!(self.stack.last(), Some(&expected));
    self.stack.pop();
  }

  pub(in crate::syntax::parse) fn current(&self) -> Option<ExprCtx> {
    self.stack.last().copied()
  }

  pub(in crate::syntax::parse) fn should_expr_span_newline(&self) -> bool {
    self.stack.iter().any(|c| c.should_expr_span_newline())
  }

  /// Find the innermost context on the stack that can handle the given token kind
  /// Falls back to the current (innermost) context if none matches
  pub(in crate::syntax::parse) fn find_handler(&self, kind: SyntaxKind) -> Option<ExprCtx> {
    self
      .stack
      .iter()
      .rev()
      .copied()
      .find(|ctx| ctx.can_handle(kind))
  }
}
