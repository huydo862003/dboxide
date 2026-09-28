pub mod cache;
pub mod node;
pub mod token;

use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};

use crate::syntax::ast::SyntaxKind;

pub use node::SyntaxNode;
pub use token::SyntaxToken;

/// GreenNode: A tagged pointer to either a Node or a Token.
/// Tag bit 0 = node, tag bit 1 = token.
pub struct GreenNode(usize);

impl GreenNode {
  pub fn from_node(node: SyntaxNode) -> Self {
    let ptr = node.0 as usize;
    debug_assert!(ptr & 1 == 0, "Node pointer not aligned");
    std::mem::forget(node);
    Self(ptr)
  }

  pub fn from_token(token: SyntaxToken) -> Self {
    let ptr = token.0 as usize;
    debug_assert!(ptr & 1 == 0, "Token pointer not aligned");
    std::mem::forget(token);
    Self(ptr | 1)
  }

  pub fn is_node(&self) -> bool {
    self.0 & 1 == 0
  }

  pub fn is_token(&self) -> bool {
    self.0 & 1 == 1
  }

  pub fn as_node(&self) -> Option<&SyntaxNode> {
    if !self.is_node() {
      return None;
    }
    unsafe { Some(&*(&self.0 as *const usize as *const SyntaxNode)) }
  }

  pub fn as_token(&self) -> Option<SyntaxToken> {
    if !self.is_token() {
      return None;
    }
    let tmp = SyntaxToken((self.0 & !1) as *const _);
    let cloned = tmp.clone();
    std::mem::forget(tmp);
    Some(cloned)
  }

  pub fn kind(&self) -> SyntaxKind {
    if self.is_token() {
      self.as_token().unwrap().kind()
    } else {
      self.as_node().unwrap().kind()
    }
  }

  pub fn text_len(&self) -> usize {
    if self.is_token() {
      self.as_token().unwrap().text_len()
    } else {
      self.as_node().unwrap().text_len()
    }
  }
}

impl Clone for GreenNode {
  fn clone(&self) -> Self {
    if self.is_node() {
      let tmp = SyntaxNode(self.0 as *const _);
      let cloned = tmp.clone();
      std::mem::forget(tmp);
      Self::from_node(cloned)
    } else {
      let tmp = SyntaxToken((self.0 & !1) as *const _);
      let cloned = tmp.clone();
      std::mem::forget(tmp);
      Self::from_token(cloned)
    }
  }
}

impl Drop for GreenNode {
  fn drop(&mut self) {
    if self.is_node() {
      drop(SyntaxNode(self.0 as *const _));
    } else {
      drop(SyntaxToken((self.0 & !1) as *const _));
    }
  }
}

impl PartialEq for GreenNode {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0 || {
      match (self.is_node(), other.is_node()) {
        (true, true) => self.as_node().unwrap() == other.as_node().unwrap(),
        (false, false) => self.as_token().unwrap() == other.as_token().unwrap(),
        _ => false,
      }
    }
  }
}

impl Eq for GreenNode {}

impl Debug for GreenNode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.is_node() {
      self.as_node().unwrap().fmt(f)
    } else {
      self.as_token().unwrap().fmt(f)
    }
  }
}

impl Hash for GreenNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    if self.is_node() {
      0u8.hash(state);
      self.as_node().unwrap().hash(state);
    } else {
      1u8.hash(state);
      self.as_token().unwrap().hash(state);
    }
  }
}

unsafe impl Send for GreenNode {}
unsafe impl Sync for GreenNode {}
