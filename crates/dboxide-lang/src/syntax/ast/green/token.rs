use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::cache::Cache;
use crate::syntax::ast::SyntaxKind;

pub(super) struct TokenBody {
  pub(super) ref_count: AtomicUsize,
  pub(super) kind: SyntaxKind,
  pub(super) bytes: Vec<u8>,
}

/// The leaf node in the green tree
pub struct SyntaxToken(pub(super) *const TokenBody);

impl SyntaxToken {
  pub(crate) fn new(cache: Rc<RefCell<Cache>>, kind: SyntaxKind, text: &[u8]) -> Self {
    cache.borrow_mut().token(kind, text)
  }

  pub(crate) fn from_raw_parts(kind: SyntaxKind, bytes: Vec<u8>) -> Self {
    let body = Box::new(TokenBody {
      ref_count: AtomicUsize::new(1),
      kind,
      bytes,
    });
    Self(Box::into_raw(body))
  }

  pub fn kind(&self) -> SyntaxKind {
    unsafe { (*self.0).kind }
  }

  pub fn text(&self) -> Option<&str> {
    unsafe { std::str::from_utf8(&(*self.0).bytes).ok() }
  }

  pub fn bytes(&self) -> &[u8] {
    unsafe { &(*self.0).bytes }
  }

  pub fn text_len(&self) -> usize {
    unsafe { (*self.0).bytes.len() }
  }
}

impl Clone for SyntaxToken {
  fn clone(&self) -> Self {
    unsafe { (*self.0).ref_count.fetch_add(1, Ordering::AcqRel) };
    Self(self.0)
  }
}

impl Drop for SyntaxToken {
  fn drop(&mut self) {
    let prev = unsafe { (*self.0).ref_count.fetch_sub(1, Ordering::AcqRel) };
    if prev != 1 {
      return;
    }
    unsafe { drop(Box::from_raw(self.0 as *mut TokenBody)) };
  }
}

impl PartialEq for SyntaxToken {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0 || (self.kind() == other.kind() && self.bytes() == other.bytes())
  }
}

impl Eq for SyntaxToken {}

impl std::fmt::Debug for SyntaxToken {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}({:?})", self.kind(), self.text().unwrap_or("?"))
  }
}

impl Hash for SyntaxToken {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.kind().hash(state);
    self.bytes().hash(state);
  }
}

unsafe impl Send for SyntaxToken {}
unsafe impl Sync for SyntaxToken {}
