use salsa::interned;

use crate::ast::RedNode;
use crate::db::types::input::File;

#[interned]
pub struct Symbol<'db> {
  #[returns(ref)]
  pub kind: SymbolKind,
  #[returns(deref)]
  pub name: String,
  /// Fully qualified path, e.g. "@metatype::string"
  #[returns(deref)]
  pub definition_id: String,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
  VirtualTyp(VirtualTypKind),
  VirtualModule(VirtualModuleKind),
  VirtualFunction(VirtualFunctionKind),
  UserTyp(File, RedNode),
  UserModule(File, RedNode),
  UserFunction(File, RedNode),
  UserOperator(File, RedNode),
  UserElement(File, RedNode),
}

impl SymbolKind {
  pub fn is_virtual(&self) -> bool {
    matches!(
      self,
      SymbolKind::VirtualTyp(_)
        | SymbolKind::VirtualModule(_)
        | SymbolKind::VirtualFunction(_)
    )
  }

  pub fn is_user_defined(&self) -> bool {
    !self.is_virtual()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VirtualModuleKind {
  Metatype,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VirtualTypKind {
  MetatypeString,
  MetatypeInteger,
  MetatypeFloat,
  MetatypeBool,
  MetatypeNull,
  MetatypeColor,
  MetatypeAny,
  /// Opaque string, passed through verbatim
  MetatypeOString,
  /// `declaration[T]`
  MetatypeDeclaration,
  /// `qualified_declaration[T]`
  MetatypeQualifiedDeclaration,
  /// `reference[T]`
  MetatypeReference,
  /// `qualified_reference[T]`
  MetatypeQualifiedReference,
  /// `Map[K, V]`
  MetatypeMap,
  /// `List[V]`
  MetatypeList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VirtualFunctionKind {
  Interpret,
  Typeof,
}
