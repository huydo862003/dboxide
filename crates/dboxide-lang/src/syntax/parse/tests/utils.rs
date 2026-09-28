use std::cell::RefCell;
use std::rc::Rc;

use crate::syntax::ast::GreenNode;
use crate::syntax::ast::cache::Cache;
use crate::syntax::parse::ParseCtx;
use crate::types::PeekableStream;
use crate::types::diagnostics::Diagnostic;

pub(crate) fn parse_expr(input: &str) -> String {
  let cache = Rc::new(RefCell::new(Cache::new()));
  let stream: PeekableStream<'_, char> =
    itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
  let mut ctx = ParseCtx::new(stream, cache);
  let (node, _) = ctx.expr();
  render_tree(&node)
}

pub(crate) fn parse_expr_with_diagnostics(input: &str) -> (String, Vec<Diagnostic>) {
  let cache = Rc::new(RefCell::new(Cache::new()));
  let stream: PeekableStream<'_, char> =
    itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
  let mut ctx = ParseCtx::new(stream, cache);
  let (node, _) = ctx.expr();
  let diagnostics = ctx.diagnostics;
  (render_tree(&node), diagnostics)
}

pub(crate) fn parse_source(input: &str) -> String {
  let cache = Rc::new(RefCell::new(Cache::new()));
  let stream: PeekableStream<'_, char> =
    itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
  let mut ctx = ParseCtx::new(stream, cache);
  let node = ctx.source_file();
  render_tree(&node)
}

pub(crate) fn parse_source_with_diagnostics(input: &str) -> (String, Vec<Diagnostic>) {
  let cache = Rc::new(RefCell::new(Cache::new()));
  let stream: PeekableStream<'_, char> =
    itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
  let mut ctx = ParseCtx::new(stream, cache);
  let node = ctx.source_file();
  let diagnostics = ctx.diagnostics;
  (render_tree(&node), diagnostics)
}

/// Render a green tree in multiline lisp format
pub(crate) fn render_tree(node: &GreenNode) -> String {
  fn render_tree_inner(node: &GreenNode, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    if node.is_token() {
      let token = node.as_token().unwrap();
      let text = token.text().unwrap_or("");
      format!("{}{:?}", pad, text)
    } else {
      let node = node.as_node().unwrap();
      let children = node.children();
      if children.is_empty() {
        format!("{}({:?})", pad, node.kind())
      } else {
        let inner: Vec<String> = children
          .iter()
          .map(|c| render_tree_inner(c, indent + 1))
          .collect();
        format!("{}({:?}\n{})", pad, node.kind(), inner.join("\n"))
      }
    }
  }

  render_tree_inner(node, 0)
}
