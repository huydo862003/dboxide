use salsa::Database;

use crate::ast::RedNode;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{File, Symbol};

use super::posttype_resolve_name::resolve_name;

// resolve a reference expression to a symbol
// for ref[T]: search scope for elements whose type is T
// for qualified_reference[T]: support schema.table.column traversal
pub fn resolve_reference<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
  _target_typ: Option<LazyTyp<'db>>,
) -> Option<Symbol<'db>> {
  // for now, delegate to resolve_name
  // TODO: filter by target type T when resolving ref[T]
  *resolve_name(db, file, node, name)
}

// resolve a qualified reference like `users.id` where `users` is a qualified_declaration
// and `id` is a declaration in users' namespace
pub fn resolve_qualified_reference<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  parts: &[String],
) -> Option<Symbol<'db>> {
  if parts.is_empty() {
    return None;
  }

  let mut current = (*resolve_name(db, file, node.clone(), parts[0].clone()))?;

  for part in &parts[1..] {
    let namespace =
      crate::db::pretype::pretype_get_symbol_namespace::get_symbol_namespace(db, current);
    current = *namespace.lookup_by_name(part).next()?.1;
  }

  Some(current)
}
