use crate::db::pretype_analyze::pretype_get_file_symbols;

use super::*;

#[test]
fn reuse_wildcard_exports_transitively() {
  let db = DboxideDatabase::default();
  let dep_a = create_file(&db, "a.dbml", "type Foo {}", HashMap::new());
  let dep_b = create_file(
    &db,
    "b.dbml",
    "reuse * from './a'",
    HashMap::from([("./a".to_string(), Either::Right(dep_a))]),
  );
  let file_c = create_file(
    &db,
    "c.dbml",
    "use * from './b'",
    HashMap::from([("./b".to_string(), Either::Right(dep_b))]),
  );
  let members = get_file_scope_members(&db, file_c);
  assert_contains_symbol(&members, "Foo");
}

#[test]
fn reuse_selective_exports_transitively() {
  let db = DboxideDatabase::default();
  let dep_a = create_file(&db, "a.dbml", "type Foo {}\ntype Bar {}", HashMap::new());
  let dep_b = create_file(
    &db,
    "b.dbml",
    "reuse { type Foo } from './a'",
    HashMap::from([("./a".to_string(), Either::Right(dep_a))]),
  );
  let file_c = create_file(
    &db,
    "c.dbml",
    "use * from './b'",
    HashMap::from([("./b".to_string(), Either::Right(dep_b))]),
  );
  let members = get_file_scope_members(&db, file_c);
  assert_contains_symbol(&members, "Foo");
  assert_missing_symbol(&members, "Bar");
}

#[test]
fn reuse_use_does_not_export() {
  let db = DboxideDatabase::default();
  let dep_a = create_file(&db, "a.dbml", "type Foo {}", HashMap::new());
  let dep_b = create_file(
    &db,
    "b.dbml",
    "use * from './a'",
    HashMap::from([("./a".to_string(), Either::Right(dep_a))]),
  );
  let usable = pretype_get_file_symbols(&db, dep_b, true);
  assert_missing_symbol(usable, "Foo");
}

#[test]
fn reuse_use_is_not_transitive_through_chain() {
  let db = DboxideDatabase::default();
  let file_c = create_file(&db, "c.dbml", "type Deep {}", HashMap::new());
  let file_b = create_file(
    &db,
    "b.dbml",
    "use * from './c'\ntype Own {}",
    HashMap::from([("./c".to_string(), Either::Right(file_c))]),
  );
  let file_a = create_file(
    &db,
    "a.dbml",
    "use * from './b'",
    HashMap::from([("./b".to_string(), Either::Right(file_b))]),
  );
  let members = get_file_scope_members(&db, file_a);
  assert_contains_symbol(&members, "Own");
  assert_missing_symbol(&members, "Deep");
}

#[test]
fn reuse_is_transitive_through_chain() {
  let db = DboxideDatabase::default();
  let file_c = create_file(&db, "c.dbml", "type Deep {}", HashMap::new());
  let file_b = create_file(
    &db,
    "b.dbml",
    "reuse * from './c'\ntype Own {}",
    HashMap::from([("./c".to_string(), Either::Right(file_c))]),
  );
  let file_a = create_file(
    &db,
    "a.dbml",
    "use * from './b'",
    HashMap::from([("./b".to_string(), Either::Right(file_b))]),
  );
  let members = get_file_scope_members(&db, file_a);
  assert_contains_symbol(&members, "Own");
  assert_contains_symbol(&members, "Deep");
}

#[test]
fn reuse_selective_use_does_not_export() {
  let db = DboxideDatabase::default();
  let dep_a = create_file(&db, "a.dbml", "type Foo {}", HashMap::new());
  let dep_b = create_file(
    &db,
    "b.dbml",
    "use { type Foo } from './a'",
    HashMap::from([("./a".to_string(), Either::Right(dep_a))]),
  );
  let usable = pretype_get_file_symbols(&db, dep_b, true);
  assert_missing_symbol(usable, "Foo");
}

#[test]
fn reuse_schema_names_are_global() {
  let db = DboxideDatabase::default();
  let dep = create_file(&db, "dep.dbml", "auth.Table roles {}", HashMap::new());
  let file = create_file(
    &db,
    "main.dbml",
    "use * from './dep'",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "auth");
}

#[test]
fn reuse_exports_schemas_transitively() {
  let db = DboxideDatabase::default();
  let dep_a = create_file(&db, "a.dbml", "auth.type Role {}", HashMap::new());
  let dep_b = create_file(
    &db,
    "b.dbml",
    "reuse * from './a'",
    HashMap::from([("./a".to_string(), Either::Right(dep_a))]),
  );
  let file_c = create_file(
    &db,
    "c.dbml",
    "use * from './b'",
    HashMap::from([("./b".to_string(), Either::Right(dep_b))]),
  );
  let members = get_file_scope_members(&db, file_c);
  assert_contains_symbol(&members, "auth");
}
