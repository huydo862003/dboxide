use std::collections::BTreeMap;

use salsa::{Database, tracked};

use super::builtins::get_module_exports;
use super::types::interned::module::Module;
use super::types::interned::symbol::SymbolKind;
use super::types::Symbol;

/// Namespace members of a symbol (what `.` traversal sees)
#[tracked]
pub fn get_namespace_members<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> BTreeMap<String, Symbol<'db>> {
  match symbol.kind(db) {
    SymbolKind::VirtualModule(kind) => get_module_exports(db, Module::Virtual(*kind)),
    // TODO: UserModule -> get_module_exports(db, Module::File(file))
    // TODO: UserTyp with `namespace {}` block
    // TODO: UserElement with qualified_declaration
    _ => BTreeMap::new(),
  }
}
