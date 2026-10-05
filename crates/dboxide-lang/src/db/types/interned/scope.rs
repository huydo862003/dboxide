use salsa::interned;

use crate::ast::RedNode;
use crate::db::types::input::File;
use crate::db::types::interned::symbol::VirtualModuleKind;

#[interned]
pub struct Scope<'db> {
  #[returns(ref)]
  pub kind: ScopeKind,
}

/// Builtin -> VirtualModule/UserModule -> Block -> Func
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ScopeKind {
  Builtin,
  VirtualModule(VirtualModuleKind),
  UserModule(File),
  Block(File, RedNode),
  Func(File, RedNode),
}

impl ScopeKind {
  pub fn file(&self) -> Option<File> {
    match self {
      ScopeKind::Builtin | ScopeKind::VirtualModule(_) => None,
      ScopeKind::UserModule(file) | ScopeKind::Block(file, _) | ScopeKind::Func(file, _) => {
        Some(*file)
      }
    }
  }
}
