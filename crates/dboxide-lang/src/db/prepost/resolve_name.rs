use salsa::{Database, tracked};

use crate::ast::{AstNode, InfixExpr, RedNode};
use crate::db::prepost::get_scope_members::get_scope_members_posttype;
use crate::db::prepost::get_scope_members::get_scope_members_pretype;
use crate::db::prepost::get_symbol_namespace::get_symbol_namespace_posttype as posttype_get_symbol_namespace;
use crate::db::prepost::get_symbol_namespace::get_symbol_namespace_pretype;
use crate::db::types::interned::symbol::StaticScopeMembers;
use crate::db::types::{File, StaticScope, StaticScopeKind, Symbol};
use crate::get_node_scope;

// Pretype name resolution - sees only types and schemas
#[tracked]
pub fn resolve_name_pretype<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  let matches = resolve_name_with(
    db,
    file,
    &node,
    &name,
    &|db, scope| get_scope_members_pretype(db, scope).clone(),
    &|db, symbol| get_symbol_namespace_pretype(db, symbol).clone(),
  );
  matches.into_iter().next()
}

// Posttype name resolution - sees types, schemas, functions, and element instances
#[tracked]
pub fn resolve_name_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  let matches = resolve_name_with(
    db,
    file,
    &node,
    &name,
    &|db, scope| get_scope_members_posttype(db, scope).clone(),
    &|db, symbol| posttype_get_symbol_namespace(db, symbol),
  );
  matches.into_iter().next()
}

// Posttype - resolve all symbols matching a name (for reference[T] filtering)
pub fn resolve_name_all_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
) -> Vec<Symbol<'db>> {
  resolve_name_with(
    db,
    file,
    node,
    name,
    &|db, scope| get_scope_members_posttype(db, scope).clone(),
    &|db, symbol| posttype_get_symbol_namespace(db, symbol),
  )
}

fn resolve_name_with<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
  get_members_func: &dyn Fn(&'db dyn Database, StaticScope<'db>) -> StaticScopeMembers<'db>,
  get_namespace_func: &dyn Fn(&'db dyn Database, Symbol<'db>) -> StaticScopeMembers<'db>,
) -> Vec<Symbol<'db>> {
  let dot_results = resolve_dot_rhs(db, file, node, name, get_members_func, get_namespace_func);
  if !dot_results.is_empty() {
    return dot_results;
  }
  resolve_free_ident(db, file, node, name, get_members_func)
}

fn resolve_free_ident<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
  get_members_func: &dyn Fn(&'db dyn Database, StaticScope<'db>) -> StaticScopeMembers<'db>,
) -> Vec<Symbol<'db>> {
  let mut current_scope = *get_node_scope(db, file, node.clone());
  loop {
    let members = get_members_func(db, current_scope);
    let matches: Vec<Symbol<'db>> = members
      .lookup_by_name(name)
      .map(|(_, symbol)| *symbol)
      .collect();
    if !matches.is_empty() {
      return matches;
    }
    match get_parent_scope(db, current_scope) {
      Some(parent) => current_scope = parent,
      None => return Vec::new(),
    }
  }
}

fn resolve_dot_rhs<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
  get_members_func: &dyn Fn(&'db dyn Database, StaticScope<'db>) -> StaticScopeMembers<'db>,
  get_namespace_func: &dyn Fn(&'db dyn Database, Symbol<'db>) -> StaticScopeMembers<'db>,
) -> Vec<Symbol<'db>> {
  let Some(parent) = node.parent() else {
    return Vec::new();
  };
  let Some(infix) = InfixExpr::cast(parent) else {
    return Vec::new();
  };
  if infix.op().as_deref() != Some(".") {
    return Vec::new();
  }
  let Some(rhs) = infix.right() else {
    return Vec::new();
  };
  if rhs.syntax().offset() != node.offset() {
    return Vec::new();
  }
  let Some(lhs) = infix.left() else {
    return Vec::new();
  };
  let Some(lhs_name) = lhs.as_name() else {
    return Vec::new();
  };
  let lhs_symbols = resolve_free_ident(db, file, lhs.syntax(), &lhs_name, get_members_func);
  let Some(lhs_symbol) = lhs_symbols.first() else {
    return Vec::new();
  };
  let namespace = get_namespace_func(db, *lhs_symbol);
  namespace
    .lookup_by_name(name)
    .map(|(_, symbol)| *symbol)
    .collect()
}

pub fn get_parent_scope<'db>(
  db: &'db dyn Database,
  scope: StaticScope<'db>,
) -> Option<StaticScope<'db>> {
  match scope.kind(db) {
    StaticScopeKind::Builtin => None,
    StaticScopeKind::VirtualModule(_) | StaticScopeKind::UserModule(_) => {
      Some(StaticScope::new(db, StaticScopeKind::Builtin))
    }
    StaticScopeKind::ElementNamespace(file, node) | StaticScopeKind::TypNamespace(file, node) => {
      let parent = node.parent()?;
      Some(*get_node_scope(db, *file, parent))
    }
    StaticScopeKind::Func(file, _) => {
      Some(StaticScope::new(db, StaticScopeKind::UserModule(*file)))
    }
  }
}
