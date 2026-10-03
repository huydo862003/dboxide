use std::hash::{Hash, Hasher};

use salsa::tracked;

use crate::{ast::RedNode, db::types::input::File, diagnostics::Diagnostic};

#[derive(Eq, PartialEq, Clone)]
pub struct CheapRedNode {
  pub node: RedNode,
  pub file: File,
}

impl Hash for CheapRedNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.node.kind().hash(state);
    self.node.offset().hash(state);
    self.node.text().hash(state);
    self.file.hash(state);
  }
}

#[tracked]
pub struct TrackedRedNode<'db> {
  value: CheapRedNode,
}

#[tracked]
pub struct FileParseResult<'db> {
  ast: TrackedRedNode<'db>,
  diagnostics: Vec<Diagnostic>,
}
