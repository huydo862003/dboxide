use crate::parse::constants::*;
use crate::parse::expr_ctx::ExprCtx;
use crate::syntax::ast::{GreenNode, SyntaxKind};
use crate::types::diagnostics::Diagnostic;

use super::{ParseCtx, Peeker};

impl<'a> ParseCtx<'a> {
  /* Source file */

  // source_file = declaration*
  pub(in crate::syntax::parse) fn source_file(&mut self) -> GreenNode {
    self.expr_ctx_stack.enter(ExprCtx::File);
    let mut children = Vec::new();
    loop {
      self.consume_trivia(&mut children, SKIP_WCN);
      let result = self.peek(SKIP_NONE);
      match result.token.kind() {
        SyntaxKind::Eof => {
          self.advance(&mut children, SKIP_NONE);
          break;
        }
        SyntaxKind::Ident => {
          let text = result.token.text().unwrap_or("");
          let (node, early_exit) =
            if text.eq_ignore_ascii_case("use") || text.eq_ignore_ascii_case("reuse") {
              self.use_declaration()
            } else if text.eq_ignore_ascii_case("fn") {
              self.fn_declaration()
            } else if text.eq_ignore_ascii_case("type") {
              self.type_declaration()
            } else {
              self.top_element_declaration()
            };
          children.push(node);
          if early_exit.is_some() {
            self.synchronize(&mut children);
          }
        }
        _ => {
          self.emit_diagnostic(Diagnostic::UnexpectedToken {
            expected: "element declaration",
            start_offset: result.start_offset,
            end_offset: result.end_offset,
          });
          self.synchronize(&mut children);
        }
      }
    }
    self.expr_ctx_stack.exit(ExprCtx::File);
    self.emit(SyntaxKind::SourceFile, &children)
  }

  /* Use declaration */

  // use_declaration = ("use" | "reuse") (Wildcard | use_specifier_list) "from" (DqString | SqString)
  pub(in crate::syntax::parse) fn use_declaration(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      matches!(
        self.peek(SKIP_WCN).token.text(),
        Some(token) if token.eq_ignore_ascii_case("use") || token.eq_ignore_ascii_case("reuse")
      ),
      "[ParseCtx::use_declaration] Expected 'use' or 'reuse' keyword"
    );
    let mut children = Vec::new();
    self.advance(&mut children, SKIP_WCN); // "use" or "reuse"

    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::Operator && peek.token.text() == Some("*") {
      self.consume_trivia(&mut children, SKIP_WC);
      let mut wildcard_children = Vec::new();
      self.advance(&mut wildcard_children, SKIP_NONE);
      children.push(self.emit(SyntaxKind::Wildcard, &wildcard_children));
    } else {
      self.consume_trivia(&mut children, SKIP_WC);
      children.push(self.use_specifier_list(SKIP_NONE));
    }

    let peek = self.peek(SKIP_WC);
    let start_offset = peek.start_offset;
    let end_offset = peek.end_offset;
    if peek.token.kind() == SyntaxKind::Ident
      && peek
        .token
        .text()
        .map(|s| s.eq_ignore_ascii_case("from"))
        .unwrap_or(false)
    {
      self.advance(&mut children, SKIP_WC);
    } else {
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "'from'",
        start_offset,
        end_offset,
      });
    }

    let peek = self.peek(SKIP_WC);
    let start_offset = peek.start_offset;
    let end_offset = peek.end_offset;
    if matches!(
      peek.token.kind(),
      SyntaxKind::DqString | SyntaxKind::SqString
    ) {
      self.advance(&mut children, SKIP_WC);
    } else {
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "string literal path",
        start_offset,
        end_offset,
      });
    }

    (self.emit(SyntaxKind::UseDeclaration, &children), None)
  }

  // use_specifier_list = "{" use_specifier* "}"
  pub(in crate::syntax::parse) fn use_specifier_list(&mut self, skip: usize) -> GreenNode {
    debug_assert!(
      self.peek(skip).token.kind() == SyntaxKind::LBrace,
      "[ParseCtx::use_specifier_list] Expected next token to be LBrace"
    );
    self.expr_ctx_stack.enter(ExprCtx::Block);
    let mut children = Vec::new();
    let open_offset = self.offset;

    let peek = self.peek(skip);
    self.consume_trivia(&mut children, skip);
    if !self.consume(
      &mut children,
      SKIP_NONE,
      SyntaxKind::LBrace,
      Diagnostic::MissingExpectedToken {
        expected: "'{'",
        start_offset: peek.start_offset,
        end_offset: peek.end_offset,
      },
    ) {
      self.expr_ctx_stack.exit(ExprCtx::Block);
      return self.emit(SyntaxKind::UseSpecifierList, &children);
    }

    loop {
      self.consume_trivia(&mut children, SKIP_WCN);
      let peek = self.peek(SKIP_NONE);
      match peek.token.kind() {
        SyntaxKind::RBrace => {
          self.advance(&mut children, SKIP_NONE);
          break;
        }
        SyntaxKind::Eof => {
          self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
            delimiter: "{",
            open_offset,
          });
          break;
        }
        SyntaxKind::Ident
          if peek
            .token
            .text()
            .map(|s| s.eq_ignore_ascii_case("from"))
            .unwrap_or(false) =>
        {
          break;
        }
        SyntaxKind::Ident => {
          children.push(self.use_specifier(SKIP_NONE));
        }
        _ => {
          self.emit_diagnostic(Diagnostic::UnexpectedToken {
            expected: "import specifier",
            start_offset: peek.start_offset,
            end_offset: peek.end_offset,
          });
          self.synchronize_use_specifier_list(&mut children);
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::Block);
    self.emit(SyntaxKind::UseSpecifierList, &children)
  }

  fn synchronize_use_specifier_list(&mut self, children: &mut Vec<GreenNode>) {
    let peek = self.peek(SKIP_NONE);
    if peek.token.kind() == SyntaxKind::RBrace || peek.token.kind() == SyntaxKind::Eof {
      return;
    }
    let mut error_children = Vec::new();
    self.advance(&mut error_children, SKIP_NONE);
    loop {
      let peek = self.peek(SKIP_NONE);
      let kind = peek.token.kind();
      if matches!(
        kind,
        SyntaxKind::RBrace
          | SyntaxKind::Eof
          | SyntaxKind::Newline
          | SyntaxKind::DqString
          | SyntaxKind::SqString
      ) || (kind == SyntaxKind::Ident
        && peek
          .token
          .text()
          .map(|s| s.eq_ignore_ascii_case("from"))
          .unwrap_or(false))
      {
        break;
      }
      self.advance(&mut error_children, SKIP_NONE);
    }
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
  }

  // use_specifier = import_kind name ("as" alias)?
  pub(in crate::syntax::parse) fn use_specifier(&mut self, skip: usize) -> GreenNode {
    let mut children = Vec::new();
    let peek = self.peek(skip);

    if peek.token.kind() == SyntaxKind::Ident {
      let kind = self.qualified_name(SKIP_WC, SyntaxKind::UseSpecifierKind);
      children.push(self.emit(SyntaxKind::IdentExpr, &[kind]));
    } else {
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "import kind specifier (e.g. table, enum)",
        start_offset: peek.start_offset,
        end_offset: peek.end_offset,
      });
      return self.emit(SyntaxKind::UseSpecifier, &children);
    }

    let peek = self.peek(SKIP_WC);
    if matches!(peek.token.kind(), SyntaxKind::Ident | SyntaxKind::DqString) {
      let name_kind = peek.token.kind();
      let name = self.qualified_name(SKIP_WC, SyntaxKind::UseSpecifierName);
      children.push(self.ident_or_dq_expr(&[name], name_kind));
    } else {
      let start_offset = peek.start_offset;
      let end_offset = peek.end_offset;
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "element name",
        start_offset,
        end_offset,
      });
      self.synchronize_use_specifier_list(&mut children);
      return self.emit(SyntaxKind::UseSpecifier, &children);
    }

    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::Ident
      && peek
        .token
        .text()
        .map(|s| s.eq_ignore_ascii_case("as"))
        .unwrap_or(false)
    {
      self.advance(&mut children, SKIP_WC); // consume "as"
      let peek = self.peek(SKIP_WC);
      if matches!(peek.token.kind(), SyntaxKind::Ident | SyntaxKind::DqString) {
        let alias_kind = peek.token.kind();
        let alias = self.qualified_name(SKIP_WC, SyntaxKind::UseSpecifierAlias);
        children.push(self.ident_or_dq_expr(&[alias], alias_kind));
      } else {
        self.emit_diagnostic(Diagnostic::MissingExpectedToken {
          expected: "alias name after 'as'",
          start_offset: peek.start_offset,
          end_offset: peek.end_offset,
        });
        self.synchronize_use_specifier_list(&mut children);
      }
    }

    self.emit(SyntaxKind::UseSpecifier, &children)
  }

  /* Block and inline element declarations */

  pub(in crate::syntax::parse) fn top_element_declaration(
    &mut self,
  ) -> (GreenNode, Option<ExprCtx>) {
    let mut children = Vec::new();
    children.push(self.qualified_name(SKIP_NONE, SyntaxKind::ElementDeclarationTyp));

    self.element_target_fragments(&mut children, SKIP_WC);

    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::Ident
      && peek
        .token
        .text()
        .map(|s| s.eq_ignore_ascii_case("as"))
        .unwrap_or(false)
    {
      self.consume_trivia(&mut children, SKIP_WC);
      self.alias(&mut children, SKIP_NONE);
    }

    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::LBracket {
      self.consume_trivia(&mut children, SKIP_WC);
      children.push(self.setting_list(SKIP_NONE));
    }

    let peek = self.peek(SKIP_WCN);
    match peek.token.kind() {
      SyntaxKind::Colon => {
        self.consume_trivia(&mut children, SKIP_WC);
        self.consume(
          &mut children,
          SKIP_NONE,
          SyntaxKind::Colon,
          Diagnostic::MissingExpectedToken {
            expected: "':'",
            start_offset: peek.start_offset,
            end_offset: peek.end_offset,
          },
        );
        self.consume_trivia(&mut children, SKIP_WC);
        if self.is_block_element_head() {
          let (node, early_exit) = self.block_element_declaration(SKIP_NONE);
          children.push(node);
          (
            self.emit(SyntaxKind::InlineElementDeclaration, &children),
            early_exit,
          )
        } else {
          children.push(self.field_declaration(SKIP_NONE));
          (
            self.emit(SyntaxKind::InlineElementDeclaration, &children),
            None,
          )
        }
      }
      SyntaxKind::LBrace => {
        self.consume_trivia(&mut children, SKIP_WCN);
        let early_exit = self.block_element_body(&mut children, SKIP_NONE);
        (
          self.emit(SyntaxKind::BlockElementDeclaration, &children),
          early_exit,
        )
      }
      _ => {
        let start_offset = peek.start_offset;
        let end_offset = peek.end_offset;
        self.emit_diagnostic(Diagnostic::MissingExpectedToken {
          expected: "'{' or ':'",
          start_offset,
          end_offset,
        });
        (self.emit(SyntaxKind::Error, &children), None)
      }
    }
  }

  // block_element = qualified_name target? alias? setting_list? block_body
  pub(in crate::syntax::parse) fn block_element_declaration(
    &mut self,
    additional_skip: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    let skip = additional_skip | SKIP_WC;
    let mut children = Vec::new();

    self.consume_trivia(&mut children, skip);
    children.push(self.qualified_name(SKIP_NONE, SyntaxKind::ElementDeclarationTyp));

    self.element_target_fragments(&mut children, skip);

    let peek = self.peek(skip);
    if peek.token.kind() == SyntaxKind::Ident
      && peek
        .token
        .text()
        .map(|s| s.eq_ignore_ascii_case("as"))
        .unwrap_or(false)
    {
      self.consume_trivia(&mut children, skip);
      self.alias(&mut children, SKIP_NONE);
    }

    let peek = self.peek(skip);
    if peek.token.kind() == SyntaxKind::LBracket {
      self.consume_trivia(&mut children, skip);
      children.push(self.setting_list(SKIP_NONE));
    }

    self.consume_trivia(&mut children, skip);
    let early_exit = self.block_element_body(&mut children, SKIP_NONE);

    (
      self.emit(SyntaxKind::BlockElementDeclaration, &children),
      early_exit,
    )
  }

  pub(in crate::syntax::parse) fn is_block_element_head(&mut self) -> bool {
    let mut peeker = self.peeker();
    if peeker.next_token(SKIP_WC).token.kind() != SyntaxKind::Ident {
      return false;
    }

    loop {
      let result = peeker.next_token(SKIP_WC);
      match result.token.kind() {
        SyntaxKind::LBrace => return true,

        SyntaxKind::LBracket => {
          if !Self::scan_past_delimited(
            &mut peeker,
            SyntaxKind::LBracket,
            SyntaxKind::RBracket,
            SKIP_WC,
          ) {
            return false;
          }
          return peeker.next_token(SKIP_WCN).token.kind() == SyntaxKind::LBrace;
        }

        SyntaxKind::LParen => {
          if !Self::scan_past_delimited(
            &mut peeker,
            SyntaxKind::LParen,
            SyntaxKind::RParen,
            SKIP_WC,
          ) {
            return false;
          }
        }

        SyntaxKind::Newline => {
          return peeker.next_token(SKIP_WCN).token.kind() == SyntaxKind::LBrace;
        }

        SyntaxKind::Eof | SyntaxKind::RBrace => return false,

        _ => {}
      }
    }
  }

  /// Scans forward through the peek stream until a matching closing delimiter is found
  fn scan_past_delimited(
    peeker: &mut Peeker<'_, '_>,
    open: SyntaxKind,
    close: SyntaxKind,
    skip: usize,
  ) -> bool {
    let mut depth = 1;
    while depth > 0 {
      let kind = peeker.next_token(skip).token.kind();
      if kind == SyntaxKind::Eof || kind == SyntaxKind::RBrace {
        return false;
      } else if kind == open {
        depth += 1;
      } else if kind == close {
        depth -= 1;
      }
    }
    true
  }

  // block_body = "{" (trivia | attribute | declaration | field)* "}"
  pub(in crate::syntax::parse) fn block_element_body(
    &mut self,
    parent: &mut Vec<GreenNode>,
    skip: usize,
  ) -> Option<ExprCtx> {
    self.expr_ctx_stack.enter(ExprCtx::Block);
    let mut children = Vec::new();
    let open_offset = self.offset;

    let peek = self.peek(skip);
    let start_offset = peek.start_offset;
    let end_offset = peek.end_offset;
    if !self.consume(
      &mut children,
      skip,
      SyntaxKind::LBrace,
      Diagnostic::MissingExpectedToken {
        expected: "'{'",
        start_offset,
        end_offset,
      },
    ) {
      self.expr_ctx_stack.exit(ExprCtx::Block);
      parent.push(self.emit(SyntaxKind::BlockElementDeclarationBody, &children));
      return None;
    }

    let mut early_exit = None;
    loop {
      self.consume_trivia(&mut children, SKIP_WCN);
      let peek = self.peek(SKIP_NONE);
      match peek.token.kind() {
        SyntaxKind::RBrace => {
          self.advance(&mut children, SKIP_NONE);
          break;
        }
        SyntaxKind::Eof => {
          self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
            delimiter: "{",
            open_offset,
          });
          break;
        }
        _ => {
          let kind = peek.token.kind();
          if let Some(handler) = self.expr_ctx_stack.find_handler(kind)
            && handler != ExprCtx::Block
          {
            self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
              delimiter: "{",
              open_offset,
            });
            early_exit = Some(handler);
            break;
          }

          if self.is_fn_head() {
            let (node, exit) = self.fn_declaration();
            children.push(node);
            if let Some(ctx) = exit
              && ctx != ExprCtx::Block
            {
              early_exit = Some(ctx);
              break;
            }
          } else if self.is_get_head() {
            let (node, exit) = self.get_declaration();
            children.push(node);
            if let Some(ctx) = exit
              && ctx != ExprCtx::Block
            {
              early_exit = Some(ctx);
              break;
            }
          } else if self.is_type_head() {
            let (node, exit) = self.type_declaration();
            children.push(node);
            if let Some(ctx) = exit
              && ctx != ExprCtx::Block
            {
              early_exit = Some(ctx);
              break;
            }
          } else if self.is_quantifier_expr_head() {
            children.push(self.field_declaration(SKIP_NONE));
          } else if self.is_attribute_head() {
            children.push(self.attribute(SKIP_NONE));
          } else if self.is_block_element_head() {
            let (node, exit) = self.block_element_declaration(SKIP_NONE);
            children.push(node);
            if let Some(ctx) = exit
              && ctx != ExprCtx::Block
            {
              early_exit = Some(ctx);
              break;
            }
          } else {
            children.push(self.field_declaration(SKIP_NONE));
          }
        }
      }
    }
    self.expr_ctx_stack.exit(ExprCtx::Block);
    parent.push(self.emit(SyntaxKind::BlockElementDeclarationBody, &children));
    early_exit
  }

  /* Field and attribute declarations */

  // field = expr+ setting_list?
  // A leading `[...]` before any args is parsed as a ListExpr arg (rest attribute syntax)
  // A `[...]` after at least one arg is the trailing setting list
  pub(in crate::syntax::parse) fn field_declaration(&mut self, initial_skip: usize) -> GreenNode {
    self.expr_ctx_stack.enter(ExprCtx::Field);
    let mut children = Vec::new();
    let mut skip = initial_skip;
    let mut can_be_settings = false;
    loop {
      let peek = self.peek(skip);
      match peek.token.kind() {
        SyntaxKind::Newline | SyntaxKind::Eof | SyntaxKind::RBrace => break,
        SyntaxKind::LBracket if can_be_settings => {
          self.consume_trivia(&mut children, skip);
          children.push(self.setting_list(SKIP_NONE));
          break;
        }
        _ => {
          self.consume_trivia(&mut children, skip);
          let (expr_node, exit) = self.expr();
          children.push(self.emit(SyntaxKind::ElementFieldDeclarationArg, &[expr_node]));
          can_be_settings = true;
          if exit.is_some() {
            break;
          }
        }
      }
      skip = SKIP_WC;
    }
    self.expr_ctx_stack.exit(ExprCtx::Field);
    self.emit(SyntaxKind::ElementFieldDeclaration, &children)
  }

  // attribute = Ident ":" value
  fn attribute(&mut self, skip: usize) -> GreenNode {
    let mut children = Vec::new();

    let mut name_children = Vec::new();

    let mut ident_children = Vec::new();
    self.advance(&mut ident_children, skip);
    name_children.push(self.emit(SyntaxKind::IdentExpr, &ident_children));

    children.push(self.emit(SyntaxKind::ElementAttributeDeclarationName, &name_children));

    let peek = self.peek(SKIP_WC);
    let start_offset = peek.start_offset;
    let end_offset = peek.end_offset;
    self.consume_trivia(&mut children, SKIP_WC);
    self.consume(
      &mut children,
      SKIP_NONE,
      SyntaxKind::Colon,
      Diagnostic::MissingExpectedToken {
        expected: "':'",
        start_offset,
        end_offset,
      },
    );

    let (value, _) = self.expr();
    children.push(self.emit(SyntaxKind::ElementAttributeDeclarationValue, &[value]));

    self.emit(SyntaxKind::ElementAttributeDeclaration, &children)
  }

  /* Element fragments */

  // element_target_fragments = target_expr{1,2}
  fn element_target_fragments(&mut self, parent: &mut Vec<GreenNode>, skip: usize) {
    for _ in 0..2 {
      if !self.is_element_target(skip) {
        break;
      }
      self.consume_trivia(parent, skip);
      let mut fragment_children = Vec::new();
      let (expr, exit) = self.expr();
      fragment_children.push(expr);
      parent.push(self.emit(
        SyntaxKind::ElementDeclarationTargetFragment,
        &fragment_children,
      ));
      if exit.is_some() {
        break;
      }
    }
  }

  // qualified_name = (Ident | string) ("." (Ident | string))*
  pub(in crate::syntax::parse) fn qualified_name(
    &mut self,
    skip: usize,
    kind: SyntaxKind,
  ) -> GreenNode {
    let mut children = Vec::new();
    self.advance(&mut children, skip);
    loop {
      let peek = self.peek(SKIP_WC);
      if peek.token.kind() == SyntaxKind::Operator && peek.token.text() == Some(".") {
        self.advance(&mut children, SKIP_WC);
        let peek_ident = self.peek(SKIP_WC);
        if matches!(
          peek_ident.token.kind(),
          SyntaxKind::Ident | SyntaxKind::DqString
        ) {
          let mut ident_children = vec![];
          self.advance(&mut ident_children, SKIP_WC);
          let ident_expr = self.ident_or_dq_expr(&ident_children, peek_ident.token.kind());
          children.push(ident_expr);
        }
      } else {
        break;
      }
    }
    self.emit(kind, &children)
  }

  // Emits IdentExpr or DqStringExpr depending on token kind
  fn ident_or_dq_expr(&mut self, children: &[GreenNode], kind: SyntaxKind) -> GreenNode {
    let inner_kind = if kind == SyntaxKind::DqString {
      SyntaxKind::DqStringExpr
    } else {
      SyntaxKind::IdentExpr
    };
    self.emit(inner_kind, children)
  }

  // alias = "as" (Ident | DqString)
  fn alias(&mut self, parent: &mut Vec<GreenNode>, skip: usize) {
    self.advance(parent, skip); // "as"
    let peek = self.peek(SKIP_WC);
    if matches!(peek.token.kind(), SyntaxKind::Ident | SyntaxKind::DqString) {
      self.consume_trivia(parent, SKIP_WC);
      let peek = self.peek(SKIP_NONE);
      let mut expr_children = Vec::new();
      self.advance(&mut expr_children, SKIP_NONE);
      let expr = self.ident_or_dq_expr(&expr_children, peek.token.kind());
      parent.push(self.emit(SyntaxKind::ElementDeclarationAlias, &[expr]));
    } else {
      let start_offset = peek.start_offset;
      let end_offset = peek.end_offset;
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "alias name after 'as'",
        start_offset,
        end_offset,
      });
      self.synchronize_element_alias(parent);
    }
  }

  fn synchronize_element_alias(&mut self, parent: &mut Vec<GreenNode>) {
    let mut error_children = Vec::new();
    loop {
      let peek = self.peek(SKIP_WC);
      let kind = peek.token.kind();
      if kind == SyntaxKind::LBracket
        || kind == SyntaxKind::LBrace
        || kind == SyntaxKind::Colon
        || kind == SyntaxKind::Newline
        || kind == SyntaxKind::Eof
      {
        break;
      }
      self.advance(&mut error_children, SKIP_NONE);
    }
    if !error_children.is_empty() {
      parent.push(self.emit(SyntaxKind::Error, &error_children));
    }
  }

  // setting_list = "[" (setting_item ("," setting_item)*)? "]"
  pub(in crate::syntax::parse) fn setting_list(&mut self, skip: usize) -> GreenNode {
    debug_assert!(
      self.peek(skip).token.kind() == SyntaxKind::LBracket,
      "[ParseCtx::setting_list] Expected next token to be LBracket"
    );
    self.expr_ctx_stack.enter(ExprCtx::List);
    let mut children = Vec::new();
    let open_offset = self.offset;
    self.advance(&mut children, skip); // [

    loop {
      self.consume_trivia(&mut children, SKIP_WCN);
      let peek = self.peek(SKIP_NONE);
      match peek.token.kind() {
        SyntaxKind::RBracket => {
          self.advance(&mut children, SKIP_NONE);
          break;
        }
        SyntaxKind::Eof => {
          self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
            delimiter: "[",
            open_offset,
          });
          break;
        }
        SyntaxKind::Comma => {
          self.advance(&mut children, SKIP_NONE);
        }
        _ => {
          // Check if token belongs to an ancestor context
          let kind = peek.token.kind();
          if let Some(handler) = self.expr_ctx_stack.find_handler(kind)
            && handler != ExprCtx::List
          {
            self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
              delimiter: "[",
              open_offset,
            });
            break;
          }
          children.push(self.setting_item());
        }
      }
    }
    self.expr_ctx_stack.exit(ExprCtx::List);
    self.emit(SyntaxKind::SettingList, &children)
  }

  // setting_item = setting_name (":" setting_value)?
  fn setting_item(&mut self) -> GreenNode {
    let mut children = Vec::new();
    let mut name_children = Vec::new();
    loop {
      let peek = self.peek(SKIP_WC);
      match peek.token.kind() {
        SyntaxKind::Ident => {
          self.consume_trivia(&mut name_children, SKIP_WC);
          self.advance(&mut name_children, SKIP_NONE);
          let after = self.peek(SKIP_WC);
          if matches!(after.token.kind(), SyntaxKind::Ident) {
            let peek_after = self.peek_nth(1, SKIP_WC);
            if !matches!(peek_after.token.kind(), SyntaxKind::Ident,) {
              // Next word is the last one, include it
              self.consume_trivia(&mut name_children, SKIP_WC);
              self.advance(&mut name_children, SKIP_NONE);
              break;
            }
            continue;
          }
          break;
        }
        SyntaxKind::DqString => {
          self.consume_trivia(&mut name_children, SKIP_WC);
          self.advance(&mut name_children, SKIP_NONE);
          break;
        }
        _ => {
          let start_offset = peek.start_offset;
          let end_offset = peek.end_offset;
          self.emit_diagnostic(Diagnostic::UnexpectedToken {
            expected: "setting name",
            start_offset,
            end_offset,
          });
          self.synchronize(&mut name_children);
          break;
        }
      }
    }
    children.push(self.emit(SyntaxKind::SettingListItemName, &name_children));

    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::Colon {
      self.consume_trivia(&mut children, SKIP_WC);
      self.advance(&mut children, SKIP_NONE);
      let (value_expr, _) = self.expr();
      children.push(self.emit(SyntaxKind::SettingListItemValue, &[value_expr]));
    }
    self.emit(SyntaxKind::SettingListItem, &children)
  }

  /* Comma expressions */

  /* Expression */

  pub(crate) fn expr(&mut self) -> (GreenNode, Option<ExprCtx>) {
    self.pratt_expr(0, vec![])
  }

  /// Pratt parser for expressions
  pub(in crate::syntax::parse) fn pratt_expr(
    &mut self,
    min_bp: u8,
    children: Vec<GreenNode>,
  ) -> (GreenNode, Option<ExprCtx>) {
    let skip = self.expr_skip_flags();

    // 1. Prefix operators
    let peek = self.peek(skip);
    let (mut lhs, early_exit) = if peek.token.kind() == SyntaxKind::Operator {
      let op_text = peek.token.text().unwrap_or("").to_string();
      if let Some(((), right_bp)) = prefix_binding_power(&op_text) {
        let mut children = children;
        self.advance(&mut children, skip);
        let (operand, exit) = self.pratt_expr(right_bp, vec![]);
        children.push(operand);
        (self.emit(SyntaxKind::PrefixExpr, &children), exit)
      } else {
        self.primary_expr(children)
      }
    } else {
      self.primary_expr(children)
    };

    if early_exit.is_some() {
      return (lhs, early_exit);
    }

    // 2. Infix / Postfix loop
    loop {
      let peek = self.peek(self.expr_skip_flags());

      // Check for Call '(' - disallow spaces after callee in fields
      if peek.token.kind() == SyntaxKind::LParen {
        let is_in_field = self.expr_ctx_stack.current() == Some(ExprCtx::Field);
        if is_in_field && self.peek(SKIP_NONE).token.kind() != SyntaxKind::LParen {
          break;
        }
        let (left_bp, ()) = postfix_binding_power("(").expect("( is in table");
        if left_bp < min_bp {
          break;
        }
        let (node, exit) = self.call_expr(lhs);
        lhs = node;
        if exit.is_some() {
          return (lhs, exit);
        }
        continue;
      }

      // Check for Index '[' - disallow spaces between array and '['
      if peek.token.kind() == SyntaxKind::LBracket {
        if self.peek(SKIP_NONE).token.kind() != SyntaxKind::LBracket {
          break;
        }
        let (left_bp, ()) = postfix_binding_power("[").expect("[ is in table");
        if left_bp < min_bp {
          break;
        }
        let (node, exit) = self.index_expr(lhs);
        lhs = node;
        if exit.is_some() {
          return (lhs, exit);
        }
        continue;
      }

      // Check for operator (infix or postfix)
      if peek.token.kind() != SyntaxKind::Operator {
        break;
      }

      let op_text = peek.token.text().unwrap_or("").to_string();

      // Check postfix (e.g. `?` or `..` without following operand)
      if op_text == "?" {
        if let Some((left_bp, ())) = postfix_binding_power(&op_text) {
          if left_bp < min_bp {
            break;
          }
          let mut children = vec![lhs];
          self.advance(&mut children, self.expr_skip_flags());
          lhs = self.emit(SyntaxKind::PostfixExpr, &children);
          continue;
        }
      } else if op_text == ".." {
        let next_token = self.peek_nth(1, self.expr_skip_flags());
        let can_start_expr = match next_token.token.kind() {
          SyntaxKind::Number
          | SyntaxKind::Ident
          | SyntaxKind::DqString
          | SyntaxKind::SqString
          | SyntaxKind::TqString
          | SyntaxKind::OqString
          | SyntaxKind::LParen
          | SyntaxKind::LBracket => true,
          SyntaxKind::Operator => {
            let next_op = next_token.token.text().unwrap_or("");
            prefix_binding_power(next_op).is_some()
          }
          _ => false,
        };
        if !can_start_expr && let Some((left_bp, ())) = postfix_binding_power(&op_text) {
          if left_bp < min_bp {
            break;
          }
          let mut children = vec![lhs];
          self.advance(&mut children, self.expr_skip_flags());
          lhs = self.emit(SyntaxKind::PostfixExpr, &children);
          continue;
        }
      }

      // Check infix
      if let Some((left_bp, right_bp)) = infix_binding_power(&op_text) {
        if left_bp < min_bp {
          break;
        }
        let mut children = vec![lhs];
        self.advance(&mut children, self.expr_skip_flags());
        let (rhs, exit) = self.pratt_expr(right_bp, vec![]);
        children.push(rhs);

        if op_text == "=>" {
          // Closure expression
          let lhs_kind = children[0].kind();
          if lhs_kind == SyntaxKind::ParenExpr {
            let node = children[0].as_node().unwrap();
            children[0] = self.emit(SyntaxKind::TupleExpr, node.children());
          }
          lhs = self.emit(SyntaxKind::ClosureExpr, &children);
        } else {
          lhs = self.emit(SyntaxKind::InfixExpr, &children);
        }

        if exit.is_some() {
          return (lhs, exit);
        }
        continue;
      }

      // Unrecognized operator
      break;
    }

    (lhs, None)
  }

  /// Parse a primary expression: literal, ident, paren, tuple, list, etc
  pub(in crate::syntax::parse) fn primary_expr(
    &mut self,
    children: Vec<GreenNode>,
  ) -> (GreenNode, Option<ExprCtx>) {
    let skip = self.expr_skip_flags();
    let peek = self.peek(skip);

    match peek.token.kind() {
      SyntaxKind::Number => {
        let mut children = children;
        self.advance(&mut children, skip);
        (self.emit(SyntaxKind::NumberExpr, &children), None)
      }
      SyntaxKind::DqString => {
        let mut children = children;
        self.advance(&mut children, skip);
        (self.emit(SyntaxKind::DqStringExpr, &children), None)
      }
      SyntaxKind::SqString => {
        let mut children = children;
        self.advance(&mut children, skip);
        (self.emit(SyntaxKind::SqStringExpr, &children), None)
      }
      SyntaxKind::TqString => {
        let mut children = children;
        self.advance(&mut children, skip);
        (self.emit(SyntaxKind::TqStringExpr, &children), None)
      }
      SyntaxKind::OqString => {
        let mut children = children;
        self.advance(&mut children, skip);
        (self.emit(SyntaxKind::OqStringExpr, &children), None)
      }
      SyntaxKind::Ident => {
        let text = peek.token.text().unwrap_or("").to_string();
        if text.eq_ignore_ascii_case("forall") {
          self.quantifier_expr(SyntaxKind::ForallExpr, children, skip)
        } else if text.eq_ignore_ascii_case("exists") {
          self.quantifier_expr(SyntaxKind::ExistsExpr, children, skip)
        } else {
          let mut children = children;
          self.advance(&mut children, skip);
          (self.emit(SyntaxKind::IdentExpr, &children), None)
        }
      }
      SyntaxKind::LParen => self.paren_or_tuple_expr(children),
      SyntaxKind::LBracket => self.list_expr(children),
      _ => {
        let handler = self.expr_ctx_stack.find_handler(peek.token.kind());
        if handler.is_some() {
          self.diagnostics.push(Diagnostic::MissingSyntaxNode {
            expected: SyntaxKind::IdentExpr,
            start_offset: self.offset,
            end_offset: self.offset,
          });
          (self.emit(SyntaxKind::Error, &children), handler)
        } else {
          let mut children = children;
          self.advance(&mut children, skip);
          let bad = children.pop().unwrap();
          children.push(self.emit(SyntaxKind::Error, &[bad]));
          self.diagnostics.push(Diagnostic::MissingSyntaxNode {
            expected: SyntaxKind::IdentExpr,
            start_offset: self.offset,
            end_offset: self.offset,
          });
          (self.emit(SyntaxKind::Error, &children), None)
        }
      }
    }
  }

  /// Parse a parenthesized or tuple expression: `(expr)` or `(expr, expr, ...)` or `()`
  pub(in crate::syntax::parse) fn paren_or_tuple_expr(
    &mut self,
    children: Vec<GreenNode>,
  ) -> (GreenNode, Option<ExprCtx>) {
    debug_assert_eq!(
      self.peek(self.expr_skip_flags()).token.kind(),
      SyntaxKind::LParen
    );
    let mut children = children;
    self.expr_ctx_stack.enter(ExprCtx::Tuple);
    let offset = self.offset;
    self.consume(
      &mut children,
      self.expr_skip_flags(),
      SyntaxKind::LParen,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::LParen,
        start_offset: offset,
        end_offset: self.offset,
      },
    );

    // Empty parens `()` -> TupleExpr
    let peek = self.peek(SKIP_ALL_TRIVIA);
    if peek.token.kind() == SyntaxKind::RParen {
      self.advance(&mut children, SKIP_ALL_TRIVIA);
      self.expr_ctx_stack.exit(ExprCtx::Tuple);
      return (self.emit(SyntaxKind::TupleExpr, &children), None);
    }

    // Parse first expression
    let (first, early_exit) = self.expr();
    children.push(first);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::Tuple) {
      self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
        delimiter: "(",
        open_offset: offset,
      });
      self.expr_ctx_stack.exit(ExprCtx::Tuple);
      return (self.emit(SyntaxKind::ParenExpr, &children), early_exit);
    }

    // If comma follows -> tuple, delegate to shared helper for remaining items
    let peek = self.peek(SKIP_ALL_TRIVIA);
    if peek.token.kind() == SyntaxKind::Comma {
      // Already have first item, parse comma + rest via shared loop
      loop {
        let peek = self.peek(SKIP_ALL_TRIVIA);
        match peek.token.kind() {
          SyntaxKind::RParen => {
            self.advance(&mut children, SKIP_ALL_TRIVIA);
            break;
          }
          SyntaxKind::Comma => {
            self.advance(&mut children, SKIP_ALL_TRIVIA);
            if self.peek(SKIP_ALL_TRIVIA).token.kind() == SyntaxKind::RParen {
              self.advance(&mut children, SKIP_ALL_TRIVIA);
              break;
            }
            let (item, exit) = self.expr();
            children.push(item);
            if exit.is_some_and(|c| c != ExprCtx::Tuple) {
              self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
                delimiter: "(",
                open_offset: offset,
              });
              self.expr_ctx_stack.exit(ExprCtx::Tuple);
              return (self.emit(SyntaxKind::TupleExpr, &children), exit);
            }
          }
          SyntaxKind::Eof => {
            self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
              delimiter: "(",
              open_offset: offset,
            });
            break;
          }
          _ => {
            let handler = self.expr_ctx_stack.find_handler(peek.token.kind());
            if handler.is_some_and(|c| c != ExprCtx::Tuple) {
              self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
                delimiter: "(",
                open_offset: offset,
              });
              self.expr_ctx_stack.exit(ExprCtx::Tuple);
              return (self.emit(SyntaxKind::TupleExpr, &children), handler);
            }
            if let Some(exit) =
              self.synchronize_delimited(ExprCtx::Tuple, SyntaxKind::RParen, &mut children)
            {
              self.expr_ctx_stack.exit(ExprCtx::Tuple);
              return (self.emit(SyntaxKind::TupleExpr, &children), Some(exit));
            }
          }
        }
      }
      self.expr_ctx_stack.exit(ExprCtx::Tuple);
      return (self.emit(SyntaxKind::TupleExpr, &children), None);
    }

    // Single expression in parens -> ParenExpr
    let offset = self.offset;
    self.consume(
      &mut children,
      SKIP_ALL_TRIVIA,
      SyntaxKind::RParen,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::RParen,
        start_offset: offset,
        end_offset: self.offset,
      },
    );
    self.expr_ctx_stack.exit(ExprCtx::Tuple);
    (self.emit(SyntaxKind::ParenExpr, &children), None)
  }

  /// Parse a list expression: `[expr, expr, ...]`
  pub(in crate::syntax::parse) fn list_expr(
    &mut self,
    children: Vec<GreenNode>,
  ) -> (GreenNode, Option<ExprCtx>) {
    debug_assert_eq!(
      self.peek(self.expr_skip_flags()).token.kind(),
      SyntaxKind::LBracket
    );
    let mut children = children;
    self.expr_ctx_stack.enter(ExprCtx::List);
    let offset = self.offset;
    self.consume(
      &mut children,
      self.expr_skip_flags(),
      SyntaxKind::LBracket,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::LBracket,
        start_offset: offset,
        end_offset: self.offset,
      },
    );
    let early_exit = self.parse_comma_delimited(&mut children, ExprCtx::List, SyntaxKind::RBracket);
    self.expr_ctx_stack.exit(ExprCtx::List);
    (self.emit(SyntaxKind::ListExpr, &children), early_exit)
  }

  /// Parse a function call expression: `callee(arg1, arg2, ...)`
  fn call_expr(&mut self, callee: GreenNode) -> (GreenNode, Option<ExprCtx>) {
    debug_assert_eq!(
      self.peek(self.expr_skip_flags()).token.kind(),
      SyntaxKind::LParen
    );
    let mut children = vec![callee];
    self.expr_ctx_stack.enter(ExprCtx::Tuple);
    let offset = self.offset;
    self.consume(
      &mut children,
      self.expr_skip_flags(),
      SyntaxKind::LParen,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::LParen,
        start_offset: offset,
        end_offset: self.offset,
      },
    );
    let early_exit = self.parse_comma_delimited(&mut children, ExprCtx::Tuple, SyntaxKind::RParen);
    self.expr_ctx_stack.exit(ExprCtx::Tuple);
    (self.emit(SyntaxKind::CallExpr, &children), early_exit)
  }

  /// Parse an index expression: `target[arg1, arg2, ...]`
  fn index_expr(&mut self, target: GreenNode) -> (GreenNode, Option<ExprCtx>) {
    debug_assert_eq!(
      self.peek(self.expr_skip_flags()).token.kind(),
      SyntaxKind::LBracket
    );
    let mut children = vec![target];
    self.expr_ctx_stack.enter(ExprCtx::List);
    let offset = self.offset;
    self.consume(
      &mut children,
      self.expr_skip_flags(),
      SyntaxKind::LBracket,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::LBracket,
        start_offset: offset,
        end_offset: self.offset,
      },
    );
    let early_exit = self.parse_comma_delimited(&mut children, ExprCtx::List, SyntaxKind::RBracket);
    self.expr_ctx_stack.exit(ExprCtx::List);
    (self.emit(SyntaxKind::IndexExpr, &children), early_exit)
  }

  /// Synchronize inside a delimited context (list or tuple)
  fn synchronize_delimited(
    &mut self,
    ctx: ExprCtx,
    closer: SyntaxKind,
    children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.peek(SKIP_NONE);
      match peek.token.kind() {
        kind if kind == SyntaxKind::Comma || kind == closer || kind == SyntaxKind::Eof => {
          break None;
        }
        _ => {
          let handler = self.expr_ctx_stack.find_handler(peek.token.kind());
          if handler.is_some_and(|c| c != ctx) {
            break handler;
          }
          self.advance(&mut error_children, SKIP_NONE);
        }
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse a comma-separated list inside delimiters: open (item ("," item)*)? close
  /// Used by call_expr, index_expr, paren_or_tuple_expr, list_expr
  fn parse_comma_delimited(
    &mut self,
    children: &mut Vec<GreenNode>,
    ctx: ExprCtx,
    closer: SyntaxKind,
  ) -> Option<ExprCtx> {
    // Empty: closer immediately
    let peek = self.peek(SKIP_ALL_TRIVIA);
    if peek.token.kind() == closer {
      self.advance(children, SKIP_ALL_TRIVIA);
      return None;
    }

    // First item
    let (first, early_exit) = self.expr();
    children.push(first);
    if early_exit.is_some_and(|c| c != ctx) {
      return early_exit;
    }

    // Remaining items
    loop {
      let peek = self.peek(SKIP_ALL_TRIVIA);
      match peek.token.kind() {
        kind if kind == closer => {
          self.advance(children, SKIP_ALL_TRIVIA);
          break;
        }
        SyntaxKind::Comma => {
          self.advance(children, SKIP_ALL_TRIVIA);
          let peek_after = self.peek(SKIP_ALL_TRIVIA);
          if peek_after.token.kind() == closer {
            self.advance(children, SKIP_ALL_TRIVIA);
            break;
          }
          let (item, early_exit) = self.expr();
          children.push(item);
          if early_exit.is_some_and(|c| c != ctx) {
            return early_exit;
          }
        }
        SyntaxKind::Eof => {
          self.diagnostics.push(Diagnostic::MissingSyntaxNode {
            expected: closer,
            start_offset: self.offset,
            end_offset: self.offset,
          });
          break;
        }
        _ => {
          let handler = self.expr_ctx_stack.find_handler(peek.token.kind());
          if handler.is_some_and(|c| c != ctx) {
            return handler;
          }
          if let Some(exit) = self.synchronize_delimited(ctx, closer, children) {
            return Some(exit);
          }
        }
      }
    }
    None
  }

  /* fn / get / type declarations */

  pub(in crate::syntax::parse) fn fn_declaration(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mut children = Vec::new();
    self.advance(&mut children, SKIP_NONE); // "fn"

    // name: "operator<sym>" or plain ident
    self.consume_trivia(&mut children, SKIP_WC);
    let mut name_children = Vec::new();
    let peek = self.peek(SKIP_NONE);
    if peek.token.kind() == SyntaxKind::Ident {
      let is_operator_kw = peek
        .token
        .text()
        .map(|t| t.eq_ignore_ascii_case("operator"))
        .unwrap_or(false);
      self.advance(&mut name_children, SKIP_NONE);
      if is_operator_kw {
        // consume the operator symbol (allow optional whitespace)
        let peek2 = self.peek(SKIP_WC);
        if peek2.token.kind() == SyntaxKind::Operator {
          self.consume_trivia(&mut name_children, SKIP_WC);
          self.advance(&mut name_children, SKIP_NONE);
        } else {
          self.emit_diagnostic(Diagnostic::MissingExpectedToken {
            expected: "operator symbol after 'operator'",
            start_offset: peek2.start_offset,
            end_offset: peek2.end_offset,
          });
        }
      }
    } else {
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "function name or 'operator<sym>'",
        start_offset: peek.start_offset,
        end_offset: peek.end_offset,
      });
    }
    children.push(self.emit(SyntaxKind::FuncDeclarationName, &name_children));

    // optional params
    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::LParen {
      self.consume_trivia(&mut children, SKIP_WC);
      children.push(self.fn_declaration_params());
    }

    // optional return type
    let peek = self.peek(SKIP_WCN);
    if peek.token.kind() == SyntaxKind::Colon {
      self.consume_trivia(&mut children, SKIP_WCN);
      children.push(self.fn_return_type());
    }

    // block body
    let peek = self.peek(SKIP_WCN);
    if peek.token.kind() == SyntaxKind::LBrace {
      self.consume_trivia(&mut children, SKIP_WCN);
      let early_exit = self.block_element_body(&mut children, SKIP_NONE);
      return (
        self.emit(SyntaxKind::FuncDeclaration, &children),
        early_exit,
      );
    }
    self.emit_diagnostic(Diagnostic::MissingExpectedToken {
      expected: "'{'",
      start_offset: peek.start_offset,
      end_offset: peek.end_offset,
    });
    (self.emit(SyntaxKind::FuncDeclaration, &children), None)
  }

  pub(in crate::syntax::parse) fn get_declaration(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mut children = Vec::new();
    self.advance(&mut children, SKIP_NONE); // "get"

    // name
    self.consume_trivia(&mut children, SKIP_WC);
    let mut name_children = Vec::new();
    let peek = self.peek(SKIP_NONE);
    if peek.token.kind() == SyntaxKind::Ident {
      self.advance(&mut name_children, SKIP_NONE);
    } else {
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "getter name",
        start_offset: peek.start_offset,
        end_offset: peek.end_offset,
      });
    }
    children.push(self.emit(SyntaxKind::GetDeclarationName, &name_children));

    // optional params
    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::LParen {
      self.consume_trivia(&mut children, SKIP_WC);
      children.push(self.fn_declaration_params());
    }

    // optional return type
    let peek = self.peek(SKIP_WCN);
    if peek.token.kind() == SyntaxKind::Colon {
      self.consume_trivia(&mut children, SKIP_WCN);
      children.push(self.fn_return_type());
    }

    // block body
    let peek = self.peek(SKIP_WCN);
    if peek.token.kind() == SyntaxKind::LBrace {
      self.consume_trivia(&mut children, SKIP_WCN);
      let early_exit = self.block_element_body(&mut children, SKIP_NONE);
      return (self.emit(SyntaxKind::GetDeclaration, &children), early_exit);
    }
    self.emit_diagnostic(Diagnostic::MissingExpectedToken {
      expected: "'{'",
      start_offset: peek.start_offset,
      end_offset: peek.end_offset,
    });
    (self.emit(SyntaxKind::GetDeclaration, &children), None)
  }

  pub(in crate::syntax::parse) fn type_declaration(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mut children = Vec::new();

    // Wrap "type" in ElementDeclarationTyp like other element declarations
    let mut typ_children = Vec::new();
    self.advance(&mut typ_children, SKIP_NONE); // "type"
    children.push(self.emit(SyntaxKind::ElementDeclarationTyp, &typ_children));

    // name
    self.consume_trivia(&mut children, SKIP_WC);
    let mut name_children = Vec::new();
    let peek = self.peek(SKIP_NONE);
    if matches!(peek.token.kind(), SyntaxKind::Ident | SyntaxKind::DqString) {
      self.advance(&mut name_children, SKIP_NONE);
    } else {
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "type name",
        start_offset: peek.start_offset,
        end_offset: peek.end_offset,
      });
    }
    children.push(self.emit(SyntaxKind::EqualityDeclarationName, &name_children));

    // optional setting [...]
    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::LBracket {
      self.consume_trivia(&mut children, SKIP_WC);
      children.push(self.setting_list(SKIP_NONE));
    }

    // either "=" for equality or "{" for block element
    let peek = self.peek(SKIP_WCN);
    match peek.token.kind() {
      SyntaxKind::Operator if peek.token.text() == Some("=") => {
        self.consume_trivia(&mut children, SKIP_WCN);
        self.advance(&mut children, SKIP_NONE); // "="
        self.consume_trivia(&mut children, SKIP_WC);
        self.expr_ctx_stack.enter(ExprCtx::Field);
        let (rhs_expr, _) = self.expr();
        self.expr_ctx_stack.exit(ExprCtx::Field);
        children.push(rhs_expr);
        (self.emit(SyntaxKind::EqualityDeclaration, &children), None)
      }
      SyntaxKind::LBrace => {
        self.consume_trivia(&mut children, SKIP_WCN);
        let early_exit = self.block_element_body(&mut children, SKIP_NONE);
        (
          self.emit(SyntaxKind::BlockElementDeclaration, &children),
          early_exit,
        )
      }
      _ => {
        self.emit_diagnostic(Diagnostic::MissingExpectedToken {
          expected: "'{' or '='",
          start_offset: peek.start_offset,
          end_offset: peek.end_offset,
        });
        (self.emit(SyntaxKind::EqualityDeclaration, &children), None)
      }
    }
  }

  fn fn_return_type(&mut self) -> GreenNode {
    let mut rt_children = Vec::new();
    self.advance(&mut rt_children, SKIP_NONE); // ":"
    self.consume_trivia(&mut rt_children, SKIP_WC);
    self.expr_ctx_stack.enter(ExprCtx::Field);
    let (type_expr, _) = self.expr();
    self.expr_ctx_stack.exit(ExprCtx::Field);
    rt_children.push(type_expr);
    self.emit(SyntaxKind::FuncDeclarationReturnTyp, &rt_children)
  }

  fn fn_declaration_params(&mut self) -> GreenNode {
    debug_assert_eq!(
      self.peek(SKIP_NONE).token.kind(),
      SyntaxKind::LParen,
      "[ParseCtx::fn_declaration_params] Expected next token to be LParen"
    );
    let mut children = Vec::new();
    let open_offset = self.offset;
    self.advance(&mut children, SKIP_NONE); // "("
    self.expr_ctx_stack.enter(ExprCtx::Tuple);

    loop {
      self.consume_trivia(&mut children, SKIP_WCN);
      let peek = self.peek(SKIP_NONE);
      match peek.token.kind() {
        SyntaxKind::RParen => {
          self.advance(&mut children, SKIP_NONE);
          break;
        }
        SyntaxKind::Eof => {
          self.emit_diagnostic(Diagnostic::UnclosedDelimiter {
            delimiter: "(",
            open_offset,
          });
          break;
        }
        SyntaxKind::Comma => {
          self.advance(&mut children, SKIP_NONE);
        }
        SyntaxKind::Ident => {
          children.push(self.fn_declaration_param());
        }
        _ => {
          let start_offset = peek.start_offset;
          let end_offset = peek.end_offset;
          self.emit_diagnostic(Diagnostic::UnexpectedToken {
            expected: "parameter name",
            start_offset,
            end_offset,
          });
          self.advance(&mut children, SKIP_NONE);
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::Tuple);
    self.emit(SyntaxKind::FuncDeclarationParams, &children)
  }

  fn fn_declaration_param(&mut self) -> GreenNode {
    debug_assert_eq!(
      self.peek(SKIP_NONE).token.kind(),
      SyntaxKind::Ident,
      "[ParseCtx::fn_declaration_param] Expected next token to be Ident"
    );
    let mut children = Vec::new();
    // param name: wrap in IdentExpr for consistency
    let mut name_children = Vec::new();
    self.advance(&mut name_children, SKIP_NONE);
    children.push(self.emit(SyntaxKind::IdentExpr, &name_children));

    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::Colon {
      self.consume_trivia(&mut children, SKIP_WC);
      self.advance(&mut children, SKIP_NONE); // ":"
      self.consume_trivia(&mut children, SKIP_WC);
      let (type_expr, _) = self.expr();
      children.push(type_expr);
    } else {
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "':' after parameter name",
        start_offset: peek.start_offset,
        end_offset: peek.end_offset,
      });
    }
    self.emit(SyntaxKind::FuncDeclarationParam, &children)
  }

  // quantifier_expr = ("forall" | "exists") binding "of" collection body
  fn quantifier_expr(
    &mut self,
    kind: SyntaxKind,
    children: Vec<GreenNode>,
    skip: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    let mut children = children;
    self.advance(&mut children, skip); // "forall" or "exists"

    // binding name
    self.consume_trivia(&mut children, SKIP_WC);
    let peek = self.peek(SKIP_NONE);
    if peek.token.kind() == SyntaxKind::Ident {
      let mut binding_children = Vec::new();
      self.advance(&mut binding_children, SKIP_NONE);
      children.push(self.emit(SyntaxKind::IdentExpr, &binding_children));
    } else {
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "binding variable name",
        start_offset: peek.start_offset,
        end_offset: peek.end_offset,
      });
    }

    // "of" keyword
    let peek = self.peek(SKIP_WC);
    if peek.token.kind() == SyntaxKind::Ident
      && peek
        .token
        .text()
        .map(|t| t.eq_ignore_ascii_case("of"))
        .unwrap_or(false)
    {
      self.consume_trivia(&mut children, SKIP_WC);
      self.advance(&mut children, SKIP_NONE);
    } else {
      self.emit_diagnostic(Diagnostic::MissingExpectedToken {
        expected: "'of'",
        start_offset: peek.start_offset,
        end_offset: peek.end_offset,
      });
    }

    // collection expression - stops naturally before `{`
    self.consume_trivia(&mut children, SKIP_WC);
    self.expr_ctx_stack.enter(ExprCtx::Field);
    let (collection, _) = self.expr();
    self.expr_ctx_stack.exit(ExprCtx::Field);
    children.push(collection);

    // body block
    let peek = self.peek(SKIP_WCN);
    if peek.token.kind() == SyntaxKind::LBrace {
      self.consume_trivia(&mut children, SKIP_WCN);
      let early_exit = self.block_element_body(&mut children, SKIP_NONE);
      return (self.emit(kind, &children), early_exit);
    }
    self.emit_diagnostic(Diagnostic::MissingExpectedToken {
      expected: "'{'",
      start_offset: peek.start_offset,
      end_offset: peek.end_offset,
    });
    (self.emit(kind, &children), None)
  }

  /* Utils */

  pub(in crate::syntax::parse) fn is_quantifier_expr_head(&mut self) -> bool {
    self.next_is_keyword("forall") || self.next_is_keyword("exists")
  }

  pub(in crate::syntax::parse) fn is_fn_head(&mut self) -> bool {
    self.next_is_keyword("fn")
  }

  pub(in crate::syntax::parse) fn is_get_head(&mut self) -> bool {
    self.next_is_keyword("get")
  }

  pub(in crate::syntax::parse) fn is_type_head(&mut self) -> bool {
    self.next_is_keyword("type")
  }

  fn next_is_keyword(&mut self, keyword: &str) -> bool {
    let peek = self.peek(SKIP_WC);
    peek.token.kind() == SyntaxKind::Ident
      && peek
        .token
        .text()
        .map(|t| t.eq_ignore_ascii_case(keyword))
        .unwrap_or(false)
  }

  pub(in crate::syntax::parse) fn is_attribute_head(&mut self) -> bool {
    let mut peeker = self.peeker();
    peeker.next_token(SKIP_WC).token.kind() == SyntaxKind::Ident
      && peeker.next_token(SKIP_WC).token.kind() == SyntaxKind::Colon
  }

  pub(in crate::syntax::parse) fn is_element_target(&mut self, additional_skip: usize) -> bool {
    let skip = additional_skip | SKIP_WC;
    let peek = self.peek_nth(0, skip);
    if peek.token.kind() == SyntaxKind::Ident
      && peek
        .token
        .text()
        .map(|s| s.eq_ignore_ascii_case("as"))
        .unwrap_or(false)
    {
      return false;
    }
    matches!(peek.token.kind(), SyntaxKind::Ident | SyntaxKind::DqString)
  }
}

/* Operator binding powers */

pub(in crate::syntax::parse) fn prefix_binding_power(op: &str) -> Option<((), u8)> {
  let bp = match op {
    "!" | "~" | "+" | "-" | "*" => 21,
    ".." => 18,
    _ => return None,
  };
  Some(((), bp))
}

pub(in crate::syntax::parse) fn infix_binding_power(op: &str) -> Option<(u8, u8)> {
  let bp = match op {
    "." | "@" => (23, 24),
    "*" | "/" | "%" => (19, 20),
    "+" | "-" | "++" | ".." => (17, 18),
    "<:" => (15, 16),
    "==" | "!=" | "<" | ">" | "<=" | ">=" | "<>" | "->" | "<-" | "?-" | "-?" | "?-?" | "?<"
    | "<?" | "?<?" | "?>" | ">?" | "?>?" | "?<>" | "<>?" | "?<>?" => (13, 14),
    "^" => (11, 12),
    "&&" => (9, 10),
    "||" | "|" => (7, 8),
    "??" => (6, 5), // right-associative
    "=>" => (4, 3), // right-associative (closure)
    _ => (1, 2),    // user-defined custom operator (default prec 1, left-assoc)
  };
  Some(bp)
}

pub(in crate::syntax::parse) fn postfix_binding_power(op: &str) -> Option<(u8, ())> {
  let bp = match op {
    "?" => 25,
    "(" | "[" => 25,
    ".." => 17,
    _ => return None,
  };
  Some((bp, ()))
}
