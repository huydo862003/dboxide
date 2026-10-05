use std::collections::BTreeMap;

use salsa::{Database, tracked};

use crate::ast::RedNode;
use crate::syntax::ast::SyntaxKind;

use super::builtins::{get_metatype_module_symbol, get_metatype_members};
use super::types::interned::symbol::VirtualModuleKind;
use super::types::{File, Scope, ScopeKind, Symbol};

/// Find the enclosing scope for a given AST node
#[tracked]
pub fn get_node_scope<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
) -> Scope<'db> {
  let mut current = node.parent();
  while let Some(parent) = current {
    match parent.kind() {
      SyntaxKind::ClosureExpr => {
        return Scope::new(db, ScopeKind::Func(file, parent));
      }
      SyntaxKind::BlockElementDeclarationBody => {
        return Scope::new(db, ScopeKind::Block(file, parent));
      }
      SyntaxKind::SourceFile => {
        return Scope::new(db, ScopeKind::UserModule(file));
      }
      _ => current = parent.parent(),
    }
  }
  Scope::new(db, ScopeKind::UserModule(file))
}

#[tracked]
pub fn get_parent_scope<'db>(db: &'db dyn Database, scope: Scope<'db>) -> Option<Scope<'db>> {
  match scope.kind(db) {
    ScopeKind::Builtin => None,
    ScopeKind::VirtualModule(_) | ScopeKind::UserModule(_) => {
      Some(Scope::new(db, ScopeKind::Builtin))
    }
    ScopeKind::Block(file, _) | ScopeKind::Func(file, _) => {
      Some(Scope::new(db, ScopeKind::UserModule(*file)))
    }
  }
}

#[tracked]
pub fn get_scope_members<'db>(
  db: &'db dyn Database,
  scope: Scope<'db>,
) -> BTreeMap<String, Symbol<'db>> {
  match scope.kind(db) {
    ScopeKind::Builtin => BTreeMap::from([("metatype".to_string(), get_metatype_module_symbol(db))]),
    ScopeKind::VirtualModule(VirtualModuleKind::Metatype) => get_metatype_members(db),
    ScopeKind::UserModule(_file) => {
      // TODO: file declarations + merged `use` imports
      BTreeMap::new()
    }
    ScopeKind::Block(..) => {
      // TODO: block-local declarations
      BTreeMap::new()
    }
    ScopeKind::Func(..) => {
      // TODO: closure parameters
      BTreeMap::new()
    }
  }
}
