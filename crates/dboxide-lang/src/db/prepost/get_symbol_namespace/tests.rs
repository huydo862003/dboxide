use std::collections::HashMap;

use itertools::Either;

use crate::db::DboxideDatabase;
use crate::db::prepost::get_scope_members::get_scope_members_pretype;
use crate::db::prepost::get_symbol_namespace::get_symbol_namespace_pretype;
use crate::db::test_utils::*;
use crate::db::types::interned::symbol::StaticScopeMembers;
use crate::db::types::{File, StaticScope, StaticScopeKind};

fn get_file_scope_members<'db>(db: &'db DboxideDatabase, file: File) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  get_scope_members_pretype(db, scope).clone()
}

fn get_schema_namespace<'db>(
  db: &'db DboxideDatabase,
  file: File,
  name: &str,
) -> StaticScopeMembers<'db> {
  let members = get_file_scope_members(db, file);
  let schema_symbol = get_first_symbol(&members, name);
  get_symbol_namespace_pretype(db, schema_symbol).clone()
}

#[test]
fn schema_namespace_contains_types_not_elements() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "auth.type Foo {}
auth.type Bar {}
auth.Table users {}
type Baz {}",
    HashMap::new(),
  );
  let auth_namespace = get_schema_namespace(&db, file, "auth");
  assert_contains_symbol(&auth_namespace, "Foo");
  assert_contains_symbol(&auth_namespace, "Bar");
  assert_missing_symbol(&auth_namespace, "users");
  assert_missing_symbol(&auth_namespace, "Baz");
}

#[test]
fn nested_schema_namespace() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "auth.sub.type Inner {}
auth.type Outer {}",
    HashMap::new(),
  );
  let auth_namespace = get_schema_namespace(&db, file, "auth");
  assert_contains_symbol(&auth_namespace, "Outer");
  assert_contains_symbol(&auth_namespace, "sub");
  assert_missing_symbol(&auth_namespace, "Inner");

  let sub_symbol = get_first_symbol(&auth_namespace, "sub");
  let sub_namespace = get_symbol_namespace_pretype(&db, sub_symbol).clone();
  assert_contains_symbol(&sub_namespace, "Inner");
}

#[test]
fn schema_members_are_per_file() {
  let db = DboxideDatabase::default();
  let dep = create_file(&db, "dep.dbml", "auth.type Dep {}", HashMap::new());
  let file = create_file(
    &db,
    "main.dbml",
    "use * from './dep'
auth.type Own {}",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let auth_namespace = get_schema_namespace(&db, file, "auth");
  assert_contains_symbol(&auth_namespace, "Own");
  assert_missing_symbol(&auth_namespace, "Dep");
}
