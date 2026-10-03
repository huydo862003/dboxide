use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use super::green::{GreenNode, node::SyntaxNode};
use crate::syntax::ast::SyntaxKind;

#[derive(Clone)]
struct RedNodeData {
  offset: usize,
  parent: Option<Box<RedNode>>,
  green: GreenNode,
}

impl PartialEq for RedNodeData {
  fn eq(&self, other: &Self) -> bool {
    self.offset == other.offset && self.green == other.green
  }
}

impl Eq for RedNodeData {}

impl Hash for RedNodeData {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.offset.hash(state);
    self.green.hash(state);
  }
}

/// A red node wraps a green node with offset and parent pointer, giving it identity in the source text
#[derive(Clone, Eq, PartialEq)]
pub struct RedNode(RedNodeData);

impl Hash for RedNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.hash(state);
  }
}

impl Debug for RedNode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RedNode")
      .field("offset", &self.0.offset)
      .field("green", &self.0.green)
      .finish()
  }
}

impl RedNode {
  pub fn new_root(root: GreenNode) -> RedNode {
    RedNode(RedNodeData {
      offset: 0,
      parent: None,
      green: root,
    })
  }

  pub fn kind(&self) -> SyntaxKind {
    self.0.green.kind()
  }

  pub fn parent(&self) -> Option<RedNode> {
    self.0.parent.as_deref().cloned()
  }

  pub fn text(&self) -> String {
    let mut out = String::with_capacity(self.text_len());
    self.write_text(&mut out);
    out
  }

  fn write_text(&self, out: &mut String) {
    match self.0.green.as_token() {
      Some(token) => out.push_str(token.text().unwrap_or("")),
      None => {
        for child in self.children() {
          child.write_text(out);
        }
      }
    }
  }

  pub fn offset(&self) -> usize {
    self.0.offset
  }

  pub fn text_len(&self) -> usize {
    self.0.green.text_len()
  }

  pub fn is_token(&self) -> bool {
    self.0.green.is_token()
  }

  pub fn is_node(&self) -> bool {
    self.0.green.is_node()
  }

  pub fn children(&self) -> RedNodeChildren {
    let green_node = self.0.green.as_node();
    RedNodeChildren {
      parent: self.clone(),
      green_node: green_node.cloned(),
      index: 0,
      offset: self.0.offset,
    }
  }

  /// Leading trivia nodes (whitespace, newline before content)
  pub fn leading_trivia(&self) -> Vec<RedNode> {
    self
      .children()
      .take_while(|c| c.kind().is_trivia())
      .collect()
  }

  /// Trailing trivia nodes (whitespace, newline after content)
  pub fn trailing_trivia(&self) -> Vec<RedNode> {
    let children: Vec<_> = self.children().collect();
    children
      .into_iter()
      .rev()
      .take_while(|c| c.kind().is_trivia())
      .collect::<Vec<_>>()
      .into_iter()
      .rev()
      .collect()
  }

  /// Offset and length excluding leading/trailing trivia
  pub fn trimmed_range(&self) -> (usize, usize) {
    let children: Vec<_> = self.children().collect();
    let start = children
      .iter()
      .find(|c| !c.kind().is_trivia())
      .map(|c| c.offset())
      .unwrap_or(self.offset());
    let end = children
      .iter()
      .rfind(|c| !c.kind().is_trivia())
      .map(|c| c.offset() + c.text_len())
      .unwrap_or(self.offset() + self.text_len());
    (start, end - start)
  }
}

impl Deref for RedNode {
  type Target = GreenNode;

  fn deref(&self) -> &Self::Target {
    &self.0.green
  }
}

/// A lazy iterator over a RedNode's children
pub struct RedNodeChildren {
  parent: RedNode,
  green_node: Option<SyntaxNode>,
  index: usize,
  offset: usize,
}

impl Iterator for RedNodeChildren {
  type Item = RedNode;

  fn next(&mut self) -> Option<RedNode> {
    let children = self.green_node.as_ref()?.children();
    let child = children.get(self.index)?;
    let child_offset = self.offset;
    self.offset += child.text_len();
    self.index += 1;
    Some(RedNode(RedNodeData {
      offset: child_offset,
      parent: Some(Box::new(self.parent.clone())),
      green: child.clone(),
    }))
  }
}
