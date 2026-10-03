use salsa::{Database, tracked};

use crate::ast::{AstNode, CallExpr, InfixExpr, RedNode};
use crate::db::types::interned::symbol::SymbolCategoryKind;
use crate::db::types::{File, Symbol};
use crate::{
  extract_referee_name, get_node_static_scope, is_in_typ_position, is_namespace_position,
  pretype_get_scope_members, pretype_get_symbol_namespace,
};

/// Resolves the symbol a referee node refers to in pretype scope
/// Pretype resolution always requires a concrete symbol (no type-only fallback like posttype)
#[tracked]
pub fn pretype_resolve_referee<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
) -> Option<Symbol<'db>> {
  // We only resolve names in type position in this phase
  if !is_in_typ_position(&node) {
    return None;
  }

  // Compound dot expression (a.b.c): delegate to the RHS
  if let Some(infix) = InfixExpr::cast(node.clone())
    && infix.has_op(".")
  {
    return *pretype_resolve_referee(db, file, infix.right()?.syntax().clone());
  }

  let name = extract_referee_name(&node)?;

  // If we're the RHS of a dot chain, resolve the LHS then look up in its namespace
  if let Some(infix) = node.parent().and_then(InfixExpr::cast)
    && infix.has_op(".")
    && infix
      .right()
      .is_some_and(|rhs| rhs.syntax().start_offset() == node.start_offset())
  {
    // Single-path LHS resolution only - multi-path tracking as in posttype is deferred
    let lhs_symbol = (*pretype_resolve_referee(db, file, infix.left()?.syntax().clone()))?;
    let namespace = pretype_get_symbol_namespace(db, lhs_symbol);
    // Namespace qualifier: look for Schema or Typ in both orders
    return if is_namespace_position(infix.syntax()) {
      namespace
        .lookup_symbols(db, &name, SymbolCategoryKind::Schema)
        .next()
        .or_else(|| {
          namespace
            .lookup_symbols(db, &name, SymbolCategoryKind::Typ)
            .next()
        })
    } else {
      // Final type position: only Typ
      namespace
        .lookup_symbols(db, &name, SymbolCategoryKind::Typ)
        .next()
    };
  }

  // Callee of a call expression: special case, look up function symbols
  let in_callee_position = node.parent().and_then(CallExpr::cast).is_some_and(|call| {
    call
      .callee()
      .is_some_and(|callee| callee.syntax().start_offset() == node.start_offset())
  });
  if in_callee_position {
    return pretype_resolve_ident(db, file, &node, &name, SymbolCategoryKind::Function);
  }

  // Bare ident: namespace qualifier if a dot follows, final type if standalone
  if is_namespace_position(&node) {
    pretype_resolve_ident(db, file, &node, &name, SymbolCategoryKind::Schema)
      .or_else(|| pretype_resolve_ident(db, file, &node, &name, SymbolCategoryKind::Typ))
  } else {
    pretype_resolve_ident(db, file, &node, &name, SymbolCategoryKind::Typ)
  }
}

/// Walks scope levels upward and returns the first symbol matching `name` and `category`
fn pretype_resolve_ident<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  name: &str,
  category: SymbolCategoryKind,
) -> Option<Symbol<'db>> {
  let mut scope = *get_node_static_scope(db, file, node.clone());
  loop {
    let members = pretype_get_scope_members(db, scope);
    if let Some(symbol) = members.lookup_symbols(db, name, category).next() {
      return Some(symbol);
    }
    scope = scope.get_parent(db)?;
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use itertools::Either;

  use crate::ast::{AstNode, IdentExpr, RedNode, SourceFile};
  use crate::db::DboxideDatabase;
  use crate::db::test_utils::*;
  use crate::db::types::File;
  use crate::parse_file::parse_file;

  use super::pretype_resolve_referee;

  fn parse_root(db: &DboxideDatabase, file: File) -> RedNode {
    parse_file(db, file).ast(db).value(db).node.clone()
  }

  fn find_ident_last(node: &RedNode, target: &str) -> Option<RedNode> {
    let mut result = None;
    if let Some(ident) = IdentExpr::cast(node.clone())
      && ident.name() == target
    {
      result = Some(node.clone());
    }
    for child in node.children() {
      if let Some(found) = find_ident_last(&child, target) {
        result = Some(found);
      }
    }
    result
  }

  mod resolution {
    use super::*;

    #[test]
    fn resolution_field_type_ident_resolves_to_declared_typ() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.dbml",
        r#"type Foo {}
type Bar {
  field Foo
}"#,
        HashMap::new(),
      );
      let root = parse_root(&db, file);
      let foo_node = find_ident_last(&root, "Foo").unwrap();
      let symbol = *pretype_resolve_referee(&db, file, foo_node);
      assert!(symbol.is_some(), "expected Foo to resolve");
      assert_eq!(symbol.unwrap().name(&db), "Foo");
    }

    #[test]
    fn resolution_target_name_not_in_typ_position_returns_none() {
      let db = DboxideDatabase::default();
      let file = create_file(&db, "test.dbml", r#"type Foo {}"#, HashMap::new());
      let root = parse_root(&db, file);
      let source_file = SourceFile::cast(root).unwrap();
      let foo_decl = source_file
        .element_declarations()
        .find(|declaration| {
          declaration
            .target_name()
            .is_some_and(|fragment| fragment.text().trim() == "Foo")
        })
        .unwrap();
      let target_node = foo_decl.target_name().unwrap().name().unwrap().node();
      let symbol = *pretype_resolve_referee(&db, file, target_node);
      assert!(
        symbol.is_none(),
        "target name should not resolve in pretype"
      );
    }

    #[test]
    fn resolution_element_decl_typ_ident_resolves_to_typ_symbol() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.dbml",
        r#"type Table {}
Table users {}"#,
        HashMap::new(),
      );
      let root = parse_root(&db, file);
      let source_file = SourceFile::cast(root).unwrap();
      let users_decl = source_file
        .element_declarations()
        .find(|declaration| {
          declaration
            .target_name()
            .is_some_and(|fragment| fragment.text().trim() == "users")
        })
        .unwrap();
      let table_node = users_decl.typ().unwrap().name().unwrap().node();
      let symbol = *pretype_resolve_referee(&db, file, table_node);
      assert!(symbol.is_some(), "expected Table to resolve");
      assert_eq!(symbol.unwrap().name(&db), "Table");
    }

    #[test]
    fn resolution_undeclared_typ_returns_none() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.dbml",
        r#"type Bar {
  field Missing
}"#,
        HashMap::new(),
      );
      let root = parse_root(&db, file);
      let missing_node = find_ident_last(&root, "Missing").unwrap();
      let symbol = *pretype_resolve_referee(&db, file, missing_node);
      assert!(symbol.is_none(), "undeclared type should not resolve");
    }

    #[test]
    fn resolution_imported_typ_resolves_across_file_boundary() {
      let db = DboxideDatabase::default();
      let dep = create_file(&db, "dep.dbml", r#"type SharedTyp {}"#, HashMap::new());
      let file = create_file(
        &db,
        "main.dbml",
        r#"use * from './dep'
type Consumer {
  field SharedTyp
}"#,
        HashMap::from([("./dep".to_string(), Either::Right(dep))]),
      );
      let root = parse_root(&db, file);
      let shared_node = find_ident_last(&root, "SharedTyp").unwrap();
      let symbol = *pretype_resolve_referee(&db, file, shared_node);
      assert!(
        symbol.is_some(),
        "expected SharedTyp to resolve from import"
      );
      assert_eq!(symbol.unwrap().name(&db), "SharedTyp");
    }
  }
}
