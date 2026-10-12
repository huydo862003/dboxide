use salsa::{Database, tracked};

use crate::ast::RedNode;
use crate::db::types::{File, Symbol};
use crate::db::utils::resolve_name_with;

use super::pretype_get_scope_members::get_scope_types_and_schemas;

#[tracked]
pub fn resolve_name<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  let matches = resolve_name_with(db, file, &node, &name, &|db, scope| {
    get_scope_types_and_schemas(db, scope).clone()
  });
  matches.into_iter().next()
}
