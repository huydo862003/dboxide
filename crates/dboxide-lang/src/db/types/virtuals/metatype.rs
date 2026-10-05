use salsa::Database;

use crate::db::types::Symbol;
use crate::db::types::interned::symbol::*;

pub(crate) fn pretype_get_metatype_members(db: &dyn Database) -> StaticScopeMembers<'_> {
  let mut members = StaticScopeMembers::default();

  for (name, kind) in [
    ("string", VirtualTypKind::MetatypeString),
    ("integer", VirtualTypKind::MetatypeInteger),
    ("float", VirtualTypKind::MetatypeFloat),
    ("bool", VirtualTypKind::MetatypeBool),
    ("null", VirtualTypKind::MetatypeNull),
    ("color", VirtualTypKind::MetatypeColor),
    ("any", VirtualTypKind::MetatypeAny),
    ("ostring", VirtualTypKind::MetatypeOString),
    ("declaration", VirtualTypKind::MetatypeDeclaration),
    (
      "qualified_declaration",
      VirtualTypKind::MetatypeQualifiedDeclaration,
    ),
    ("reference", VirtualTypKind::MetatypeReference),
    (
      "qualified_reference",
      VirtualTypKind::MetatypeQualifiedReference,
    ),
    ("Map", VirtualTypKind::MetatypeMap),
    ("List", VirtualTypKind::MetatypeList),
  ] {
    let symbol = Symbol::new(
      db,
      SymbolKind::VirtualTyp(VirtualModuleKind::Metatype, kind),
      name.to_string(),
      format!("@metatype::{name}"),
    );
    members.insert(db, symbol);
  }

  for (name, kind) in [
    ("interpret", VirtualFunctionKind::Interpret),
    ("typeof", VirtualFunctionKind::Typeof),
  ] {
    let symbol = Symbol::new(
      db,
      SymbolKind::VirtualFunction(VirtualModuleKind::Metatype, kind),
      name.to_string(),
      format!("@metatype::{name}"),
    );
    members.insert(db, symbol);
  }

  members
}
