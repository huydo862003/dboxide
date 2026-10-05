use salsa::{Database, tracked};

use crate::ast::RedNode;
use crate::syntax::ast::SyntaxKind;

use crate::db::types::{File, StaticScope, StaticScopeKind};

/// Find the enclosing scope for a given AST node
#[tracked]
pub fn get_node_scope<'db>(db: &'db dyn Database, file: File, node: RedNode) -> StaticScope<'db> {
  let mut current = node.parent();
  while let Some(parent) = current {
    match parent.kind() {
      SyntaxKind::ClosureExpr => {
        return StaticScope::new(db, StaticScopeKind::Func(file, parent));
      }
      SyntaxKind::BlockElementDeclarationBody => {
        return StaticScope::new(db, StaticScopeKind::ElementNamespace(file, parent));
      }
      SyntaxKind::SourceFile => {
        return StaticScope::new(db, StaticScopeKind::UserModule(file));
      }
      _ => current = parent.parent(),
    }
  }
  StaticScope::new(db, StaticScopeKind::UserModule(file))
}
