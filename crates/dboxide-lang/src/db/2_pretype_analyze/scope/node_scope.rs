use salsa::{Database, tracked};

use crate::ast::{AstNode, ElementDeclaration, RedNode};
use crate::syntax::ast::SyntaxKind;

use crate::db::types::{File, StaticScope, StaticScopeKind};

/// Find the enclosing scope for a given AST node
#[tracked]
pub fn get_node_static_scope<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
) -> StaticScope<'db> {
  let mut current = node.parent();
  while let Some(parent) = current {
    match parent.kind() {
      SyntaxKind::ClosureExpr => {
        return StaticScope::new(db, StaticScopeKind::Func(file, parent));
      }
      SyntaxKind::BlockElementDeclarationBody => {
        // Distinguish type bodies from element instance bodies
        let is_typ_body = parent
          .parent()
          .and_then(ElementDeclaration::cast)
          .is_some_and(|element| element.is_typ_declaration());
        let kind = if is_typ_body {
          StaticScopeKind::TypNamespace(file, parent)
        } else {
          StaticScopeKind::ElementNamespace(file, parent)
        };
        return StaticScope::new(db, kind);
      }
      SyntaxKind::SourceFile => {
        return StaticScope::new(db, StaticScopeKind::UserModule(file));
      }
      _ => current = parent.parent(),
    }
  }
  StaticScope::new(db, StaticScopeKind::UserModule(file))
}
