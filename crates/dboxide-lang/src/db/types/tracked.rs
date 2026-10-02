use std::hash::{Hash, Hasher};

use salsa::tracked;

use crate::{
  ast::RedNode, db::types::input::File, diagnostics::Diagnostic,
};

#[derive(Eq, PartialEq, Clone)]
struct FileRedNode {
  node: RedNode,
  file: File,
}

impl Hash for FileRedNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.node.kind().hash(state);
    self.node.offset().hash(state);
    self.node.text().hash(state);
    self.file.hash(state);
  }
}

#[tracked]
struct FileParseResult<'db> {
  ast: RedNode,
  diagnostics: Vec<Diagnostic>,
}
