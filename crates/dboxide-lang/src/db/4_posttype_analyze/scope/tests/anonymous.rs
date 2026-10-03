use super::*;

use crate::db::types::interned::symbol::{SymbolKind, VirtualModuleKind};
use itertools::Either;
use std::collections::HashMap;

#[test]
fn anonymous_reference_element_gets_anonymous_symbol() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

type Ref [element] {
  from reference[Table] [arg: 0]
  to reference[Table] [arg: 1]
}

Table users {}
Table orders {}
Ref {}
"#,
    metatype(),
  );

  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "orders");
  assert!(
    members
      .iter()
      .any(|(_, symbol)| matches!(symbol.kind(&db), SymbolKind::AnonymousElement(..))),
    "expected anonymous symbol for the unlabeled Ref element"
  );
}

#[test]
fn anonymous_non_reference_element_gets_no_anonymous_symbol() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Note [element] {
  content string [arg: 0]
}

Note { 'hello' }
"#,
    metatype(),
  );

  let members = get_file_scope_members(&db, file);
  assert!(
    !members
      .iter()
      .any(|(_, symbol)| matches!(symbol.kind(&db), SymbolKind::AnonymousElement(..))),
    "non-reference unlabeled element should not produce an anonymous symbol"
  );
}

#[test]
fn nested_anonymous_reference_element_gets_anonymous_symbol() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

type Ref [element, use: follow refs] {
  from reference[Table] [arg: 0]
  to reference[Table] [arg: 1]
}

Table users {}
Table orders {}
Ref {
  users.id
  orders.id
}
"#,
    metatype(),
  );

  let members = get_file_scope_members(&db, file);
  assert!(
    members
      .iter()
      .any(|(_, symbol)| matches!(symbol.kind(&db), SymbolKind::AnonymousElement(..))),
    "expected anonymous symbol for unlabeled Ref with follow-refs policy"
  );
}

#[test]
fn nested_anonymous_element_imported_via_wildcard_when_refs_in_scope() {
  let db = DboxideDatabase::default();
  let dep = create_file(
    &db,
    "dep.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

type Ref [element, use: follow refs] {
  from reference[Table] [arg: 0]
  to reference[Table] [arg: 1]
}

Table users {}
Table orders {}
Ref {
  users.id
  orders.id
}
"#,
    metatype(),
  );
  let main = create_file(
    &db,
    "main.dbml",
    r#"use * from 'dep'"#,
    HashMap::from([
      (
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      ),
      ("dep".to_string(), Either::Right(dep)),
    ]),
  );

  let members = get_file_scope_members(&db, main);
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "orders");
  assert!(
    members
      .iter()
      .any(|(_, symbold)| matches!(symbold.kind(&db), SymbolKind::Use(inner, ..) if matches!(inner.kind(&db), SymbolKind::AnonymousElement(..)))),
    "expected anonymous Ref to be imported via wildcard when its refs are in scope"
  );
}
