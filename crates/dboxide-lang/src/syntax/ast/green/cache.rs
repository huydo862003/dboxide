use std::cell::RefCell;
use std::hash::{BuildHasher, Hash, Hasher};
use std::rc::Rc;

use hashbrown::HashMap;
use hashbrown::hash_map::RawEntryMut;

use super::GreenNode;
use super::node::SyntaxNode;
use super::token::SyntaxToken;
use crate::syntax::ast::SyntaxKind;

thread_local! {
  static CACHE: Rc<RefCell<Cache>> = Rc::new(RefCell::new(Cache::new()));
}

/// A non-thread-safe interner for node/token deduplication
#[derive(Default)]
pub struct Cache {
  tokens: HashMap<SyntaxToken, ()>,
  nodes: HashMap<SyntaxNode, ()>,
}

/// Access the thread-local green cache
pub fn with_green_cache<F, R>(f: F) -> R
where
  F: FnOnce(&mut Cache) -> R,
{
  CACHE.with(|cache| f(&mut cache.borrow_mut()))
}

/// Get a clone of the thread-local green cache handle
pub fn green_cache() -> Rc<RefCell<Cache>> {
  CACHE.with(|cache| cache.clone())
}

impl Cache {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn token(&mut self, kind: SyntaxKind, bytes: &[u8]) -> SyntaxToken {
    let hash = {
      let mut h = self.tokens.hasher().build_hasher();
      kind.hash(&mut h);
      bytes.hash(&mut h);
      h.finish()
    };

    let entry = self.tokens.raw_entry_mut().from_hash(hash, |existing| {
      existing.kind() == kind && existing.bytes() == bytes
    });

    match entry {
      RawEntryMut::Occupied(e) => e.key().clone(),
      RawEntryMut::Vacant(e) => {
        let token = SyntaxToken::from_raw_parts(kind, bytes.to_vec());
        e.insert_hashed_nocheck(hash, token.clone(), ());
        token
      }
    }
  }

  pub fn node(&mut self, kind: SyntaxKind, children: &[GreenNode]) -> SyntaxNode {
    let hash = {
      let mut h = self.nodes.hasher().build_hasher();
      kind.hash(&mut h);
      children.hash(&mut h);
      h.finish()
    };

    let entry = self.nodes.raw_entry_mut().from_hash(hash, |existing| {
      existing.kind() == kind && existing.children() == children
    });

    match entry {
      RawEntryMut::Occupied(e) => e.key().clone(),
      RawEntryMut::Vacant(e) => {
        let node = SyntaxNode::from_raw_parts(kind, children);
        e.insert_hashed_nocheck(hash, node.clone(), ());
        node
      }
    }
  }
}
