use salsa::{Database, tracked};

use crate::ast::RedNode;
use crate::db::types::{File, Symbol};
use crate::db::utils::resolve_name_with;

use super::posttype_get_scope_members::get_scope_members;

// resolve an identifier - returns the first matching symbol
// for type-filtered resolution (e.g. reference[T]), use resolve_name_all
#[tracked]
pub fn resolve_name<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  let matches = resolve_name_with(db, file, &node, &name, &|db, scope| {
    get_scope_members(db, scope).clone()
  });
  matches.into_iter().next()
}

// resolve all symbols matching a name (for reference[T] filtering by target type)
pub fn resolve_name_all<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
) -> Vec<Symbol<'db>> {
  resolve_name_with(db, file, node, name, &|db, scope| {
    get_scope_members(db, scope).clone()
  })
}
