//! Field resolution (what `@` access sees)
//!
//! `@` sees ALL fields of a typ, not just those in `namespace {}`.
//! `users@columns` reads the field directly.
//! `users.columns` goes through namespace filtering.

use std::collections::BTreeMap;

use salsa::{Database, tracked};

use super::types::Symbol;
use super::types::interned::symbol::SymbolKind;

#[tracked]
pub fn get_object_fields<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> BTreeMap<String, Symbol<'db>> {
  match symbol.kind(db) {
    SymbolKind::VirtualTyp(_) | SymbolKind::VirtualModule(_) | SymbolKind::VirtualFunction(_) => {
      BTreeMap::new()
    }
    // TODO: extract fields from typ body AST
    SymbolKind::UserTyp(..) => BTreeMap::new(),
    SymbolKind::UserModule(..) => BTreeMap::new(),
    SymbolKind::UserFunction(..) => BTreeMap::new(),
    SymbolKind::UserOperator(..) => BTreeMap::new(),
    SymbolKind::UserElement(..) => BTreeMap::new(),
  }
}
