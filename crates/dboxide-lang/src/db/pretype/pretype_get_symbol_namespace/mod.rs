mod schema;
use schema::collect_usable_members_for_schema;

use salsa::{Database, tracked};

use crate::db::types::Symbol;
use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind};

/// Namespace members of a symbol (what `.` traversal sees)
/// In this phase, schemas, types, functions, operators, and schema members are returned
#[tracked]
pub fn pretype_get_symbol_namespace<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> StaticScopeMembers<'db> {
  match symbol.kind(db) {
    SymbolKind::UserSchema(file, parent) => {
      let qualified_name = build_schema_qualified_name(db, symbol, parent);
      collect_usable_members_for_schema(db, *file, &qualified_name)
    }
    _ => StaticScopeMembers::default(),
  }
}

/// Build the full qualified name for a schema by walking parent chain
fn build_schema_qualified_name<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
  parent: &Option<Symbol<'db>>,
) -> Vec<String> {
  let mut chain = vec![symbol.name(db).to_string()];
  let mut current_parent = *parent;
  while let Some(parent_symbol) = current_parent {
    chain.push(parent_symbol.name(db).to_string());
    current_parent = match parent_symbol.kind(db) {
      SymbolKind::UserSchema(_, parent_schema) => *parent_schema,
      _ => None,
    };
  }
  chain.reverse();
  chain
}
