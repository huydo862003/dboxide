use salsa::{Database, tracked};

use crate::{
  ast::{AstNode, InfixExpr, RedNode},
  get_node_scope,
};

use super::pretype_get_scope_members::pretype_get_scope_members;
use super::pretype_get_symbol_namespace::pretype_get_symbol_namespace;
use crate::db::types::{File, StaticScope, StaticScopeKind, Symbol};

/// Resolve an identifier to a symbol
/// `@` RHS is not resolved here
/// `.` RHS is resolved for modules and schemas only (not type/element namespaces)
#[tracked]
pub fn pretype_resolve_name<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  if let Some(symbol) = resolve_dot_rhs(db, file, &node, &name) {
    return Some(symbol);
  }

  pretype_resolve_free_ident(db, file, &node, &name)
}

/// Walk the scope chain looking for a matching name
fn pretype_resolve_free_ident<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
) -> Option<Symbol<'db>> {
  let mut current_scope = *get_node_scope(db, file, node.clone());
  loop {
    let members = pretype_get_scope_members(db, current_scope);
    if let Some((_key, symbol)) = members.lookup_by_name(name).next() {
      return Some(*symbol);
    }
    current_scope = get_parent_scope(db, current_scope)?;
  }
}

/// Resolve RHS of a `.` expression via namespace lookup on the LHS
fn resolve_dot_rhs<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
) -> Option<Symbol<'db>> {
  let parent = node.parent()?;
  let infix = InfixExpr::cast(parent)?;
  if infix.op().as_deref() != Some(".") {
    return None;
  }
  let rhs = infix.right()?;
  if rhs.syntax().offset() != node.offset() {
    return None;
  }
  let lhs = infix.left()?;
  let lhs_name = lhs.as_name()?;
  let lhs_symbol = pretype_resolve_free_ident(db, file, lhs.syntax(), &lhs_name)?;
  let namespace = pretype_get_symbol_namespace(db, lhs_symbol);
  namespace
    .lookup_by_name(name)
    .next()
    .map(|(_, symbol)| *symbol)
}

/// Walk up one level in the scope chain
fn get_parent_scope<'db>(
  db: &'db dyn Database,
  scope: StaticScope<'db>,
) -> Option<StaticScope<'db>> {
  match scope.kind(db) {
    StaticScopeKind::Builtin => None,
    StaticScopeKind::VirtualModule(_) | StaticScopeKind::UserModule(_) => {
      Some(StaticScope::new(db, StaticScopeKind::Builtin))
    }
    StaticScopeKind::ElementNamespace(file, node) => {
      let parent = node.parent()?;
      Some(*get_node_scope(db, *file, parent))
    }
    StaticScopeKind::TypNamespace(file, node) => {
      let parent = node.parent()?;
      Some(*get_node_scope(db, *file, parent))
    }
    StaticScopeKind::Func(file, _) => {
      Some(StaticScope::new(db, StaticScopeKind::UserModule(*file)))
    }
  }
}
