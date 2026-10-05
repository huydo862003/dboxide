use std::collections::BTreeMap;

use salsa::Database;

use crate::db::types::Symbol;
use crate::db::types::interned::symbol::*;

pub(crate) fn get_metatype_module_symbol(db: &dyn Database) -> Symbol<'_> {
  Symbol::new(
    db,
    SymbolKind::VirtualModule(VirtualModuleKind::Metatype),
    "metatype".to_string(),
    "@metatype".to_string(),
  )
}

pub(crate) fn get_metatype_members(db: &dyn Database) -> BTreeMap<String, Symbol<'_>> {
  let mut members = BTreeMap::new();

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
    members.insert(
      name.to_string(),
      Symbol::new(
        db,
        SymbolKind::VirtualTyp(kind),
        name.to_string(),
        format!("@metatype::{name}"),
      ),
    );
  }

  for (name, kind) in [
    ("interpret", VirtualFunctionKind::Interpret),
    ("typeof", VirtualFunctionKind::Typeof),
  ] {
    members.insert(
      name.to_string(),
      Symbol::new(
        db,
        SymbolKind::VirtualFunction(kind),
        name.to_string(),
        format!("@metatype::{name}"),
      ),
    );
  }

  members
}
