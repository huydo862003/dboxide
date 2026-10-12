use salsa::{Database, tracked};

use crate::ast::{AstNode, InfixExpr, RedNode};
use crate::db::prepost::get_scope_members::{
  get_scope_members_posttype, get_scope_members_pretype,
};
use crate::db::prepost::get_symbol_namespace::{
  get_symbol_namespace_posttype, get_symbol_namespace_pretype,
};
use crate::db::types::interned::symbol::CategoryKind;
use crate::db::types::{File, StaticScope, StaticScopeKind, Symbol};
use crate::get_node_scope;

/// Resolves a type-category symbol by name in the pretype scope
#[tracked]
pub fn resolve_name_pretype<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  resolve_dot_pretype(db, file, &node, &name)
    .or_else(|| scope_walk_pretype(db, file, &node, &name, CategoryKind::Typ))
}

/// Resolves a type-category symbol by name in the posttype scope
#[tracked]
pub fn resolve_name_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  resolve_dot_posttype(db, file, &node, &name, CategoryKind::Typ)
    .or_else(|| scope_walk_posttype(db, file, &node, &name, CategoryKind::Typ))
}

/// Resolves a schema-category symbol by name in the posttype scope
#[tracked]
pub fn resolve_schema_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  scope_walk_posttype(db, file, &node, &name, CategoryKind::Schema)
}

/// Resolves an alias-category symbol by name in the posttype scope
#[tracked]
pub fn resolve_alias_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  scope_walk_posttype(db, file, &node, &name, CategoryKind::Alias)
}

/// Resolves an element-category symbol by name in the posttype scope
#[tracked]
pub fn resolve_element_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
) -> Option<Symbol<'db>> {
  resolve_dot_posttype(db, file, &node, &name, CategoryKind::Element)
    .or_else(|| scope_walk_posttype(db, file, &node, &name, CategoryKind::Element))
}

/// Resolves all element-category symbols matching a name in the posttype scope
pub fn resolve_name_all_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
) -> Vec<Symbol<'db>> {
  if let Some(symbol) = resolve_dot_posttype(db, file, node, name, CategoryKind::Element) {
    return vec![symbol];
  }
  let mut scope = *get_node_scope(db, file, node.clone());
  loop {
    let members = get_scope_members_posttype(db, scope);
    let matches: Vec<_> = members
      .lookup_symbols(db, name, CategoryKind::Element)
      .collect();
    if !matches.is_empty() {
      return matches;
    }
    match get_parent_scope(db, scope) {
      Some(parent) => scope = parent,
      None => return Vec::new(),
    }
  }
}

fn resolve_dot_pretype<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
) -> Option<Symbol<'db>> {
  let infix = InfixExpr::cast(node.parent()?)?;
  if infix.op().as_deref() != Some(".") {
    return None;
  }
  if infix.right()?.syntax().offset() != node.offset() {
    return None;
  }
  let lhs = infix.left()?;
  let lhs_name = lhs.as_name()?;
  let lhs_symbol = scope_walk_pretype(db, file, lhs.syntax(), &lhs_name, CategoryKind::Schema)?;
  get_symbol_namespace_pretype(db, lhs_symbol)
    .lookup_symbols(db, name, CategoryKind::Typ)
    .next()
}

fn resolve_dot_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
  category: CategoryKind,
) -> Option<Symbol<'db>> {
  let infix = InfixExpr::cast(node.parent()?)?;
  if infix.op().as_deref() != Some(".") {
    return None;
  }
  if infix.right()?.syntax().offset() != node.offset() {
    return None;
  }
  let lhs = infix.left()?;
  let lhs_name = lhs.as_name()?;
  let lhs_symbol = scope_walk_posttype(db, file, lhs.syntax(), &lhs_name, CategoryKind::Schema)
    .or_else(|| scope_walk_posttype(db, file, lhs.syntax(), &lhs_name, CategoryKind::Element))?;
  get_symbol_namespace_posttype(db, lhs_symbol)
    .lookup_symbols(db, name, category)
    .next()
}

fn scope_walk_pretype<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
  category: CategoryKind,
) -> Option<Symbol<'db>> {
  let mut scope = *get_node_scope(db, file, node.clone());
  loop {
    let members = get_scope_members_pretype(db, scope);
    if let Some(symbol) = members.lookup_symbols(db, name, category).next() {
      return Some(symbol);
    }
    scope = get_parent_scope(db, scope)?;
  }
}

fn scope_walk_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
  category: CategoryKind,
) -> Option<Symbol<'db>> {
  let mut scope = *get_node_scope(db, file, node.clone());
  loop {
    let members = get_scope_members_posttype(db, scope);
    if let Some(symbol) = members.lookup_symbols(db, name, category).next() {
      return Some(symbol);
    }
    scope = get_parent_scope(db, scope)?;
  }
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

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use itertools::Either;

  use crate::db::DboxideDatabase;
  use crate::db::prepost::resolve_name::{resolve_element_posttype, resolve_name_all_posttype};
  use crate::db::prepost::resolve_name_posttype;
  use crate::db::test_utils::*;
  use crate::db::types::interned::symbol::{SymbolKind, VirtualModuleKind};
  use crate::parse_file::parse_file;

  fn metatype() -> HashMap<String, Either<VirtualModuleKind, crate::db::types::File>> {
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )])
  }

  #[test]
  fn posttype_resolves_type_by_name() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let resolved = resolve_name_posttype(&db, file, root.clone(), "Table".to_string());
    assert!(resolved.is_some());
    assert!(matches!(
      resolved.unwrap().kind(&db),
      SymbolKind::UserTyp(..)
    ));
  }

  #[test]
  fn posttype_does_not_resolve_element_as_typ() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let resolved = resolve_name_posttype(&db, file, root, "users".to_string());
    assert!(
      resolved.is_none(),
      "element instance should not resolve as a type"
    );
  }

  #[test]
  fn posttype_resolves_element_instance_by_name() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
Table orders {}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let resolved = resolve_element_posttype(&db, file, root.clone(), "users".to_string());
    assert!(resolved.is_some());
    assert!(matches!(
      resolved.unwrap().kind(&db),
      SymbolKind::UserElement(..)
    ));

    let resolved = resolve_element_posttype(&db, file, root, "orders".to_string());
    assert!(resolved.is_some());
  }

  #[test]
  fn resolve_all_returns_all_elements_matching_name() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let matches = resolve_name_all_posttype(&db, file, &root, "users");
    assert!(!matches.is_empty());
    assert!(matches!(matches[0].kind(&db), SymbolKind::UserElement(..)));
  }

  #[test]
  fn element_does_not_resolve_as_typ() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let matches = resolve_name_all_posttype(&db, file, &root, "Table");
    assert!(matches.is_empty(), "type should not resolve as element");
  }
}
