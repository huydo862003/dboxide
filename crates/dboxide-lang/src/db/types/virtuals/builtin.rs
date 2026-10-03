use salsa::Database;

use crate::db::types::Symbol;
use crate::db::types::interned::symbol::*;

/// Returns the built-in scope members (virtual operators)
pub(crate) fn get_builtin_members(db: &dyn Database) -> StaticScopeMembers<'_> {
  let mut members = StaticScopeMembers::default();
  for &kind in VirtualOperatorKind::all() {
    let symbol = Symbol::new(db, SymbolKind::VirtualOperator(kind));
    members.insert(db, symbol);
  }
  members
}
