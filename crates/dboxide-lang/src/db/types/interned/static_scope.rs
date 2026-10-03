use salsa::{Database, interned};

use crate::ast::RedNode;
use crate::db::types::input::File;
use crate::db::types::interned::symbol::VirtualModuleKind;
use crate::get_node_static_scope;

#[interned]
pub struct StaticScope<'db> {
  #[returns(ref)]
  pub kind: StaticScopeKind,
}

impl<'db> StaticScope<'db> {
  pub fn get_parent(&self, db: &'db dyn Database) -> Option<StaticScope<'db>> {
    match self.kind(db) {
      StaticScopeKind::Builtin => None,
      StaticScopeKind::VirtualModule(_) | StaticScopeKind::UserModule(_) => {
        Some(StaticScope::new(db, StaticScopeKind::Builtin))
      }
      StaticScopeKind::ElementNamespace(file, node) | StaticScopeKind::TypNamespace(file, node) => {
        let parent = node.parent()?;
        Some(*get_node_static_scope(db, *file, parent))
      }
      StaticScopeKind::Func(file, node) => {
        let parent = node.parent()?;
        Some(*get_node_static_scope(db, *file, parent))
      }
    }
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum StaticScopeKind {
  Builtin,
  VirtualModule(VirtualModuleKind),
  UserModule(File),
  ElementNamespace(File, RedNode),
  TypNamespace(File, RedNode),
  Func(File, RedNode),
}

impl StaticScopeKind {
  pub fn file(&self) -> Option<File> {
    match self {
      StaticScopeKind::Builtin | StaticScopeKind::VirtualModule(_) => None,
      StaticScopeKind::UserModule(file)
      | StaticScopeKind::Func(file, _)
      | StaticScopeKind::ElementNamespace(file, _)
      | StaticScopeKind::TypNamespace(file, _) => Some(*file),
    }
  }
}
