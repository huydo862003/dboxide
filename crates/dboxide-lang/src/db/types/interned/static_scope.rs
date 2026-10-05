use salsa::interned;

use crate::ast::RedNode;
use crate::db::types::input::File;
use crate::db::types::interned::symbol::VirtualModuleKind;

#[interned]
pub struct StaticScope<'db> {
  #[returns(ref)]
  pub kind: StaticScopeKind,
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
