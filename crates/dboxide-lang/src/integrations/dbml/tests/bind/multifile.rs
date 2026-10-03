use super::*;

#[test]
fn wildcard_use_is_not_transitive() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file_c = dbml(
    &db,
    prelude,
    "c.dbml",
    r#"Table t1 { id integer [pk] }"#,
    HashMap::new(),
  );
  let file_b = dbml(
    &db,
    prelude,
    "b.dbml",
    r#"
use * from './c'
Table t2 { id integer [pk] }
"#,
    HashMap::from([("./c".to_string(), Either::Right(file_c))]),
  );
  let file_a = dbml(
    &db,
    prelude,
    "a.dbml",
    r#"use * from './b'"#,
    HashMap::from([("./b".to_string(), Either::Right(file_b))]),
  );
  let members = posttype_members(&db, file_a);
  assert_contains_symbol(&members, "t2");
  assert_missing_symbol(&members, "t1");
}

#[test]
fn wildcard_reuse_is_transitive() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file_c = dbml(
    &db,
    prelude,
    "c.dbml",
    r#"Table t1 { id integer [pk] }"#,
    HashMap::new(),
  );
  let file_b = dbml(
    &db,
    prelude,
    "b.dbml",
    r#"
reuse * from './c'
Table t2 { id integer [pk] }
"#,
    HashMap::from([("./c".to_string(), Either::Right(file_c))]),
  );
  let file_a = dbml(
    &db,
    prelude,
    "a.dbml",
    r#"use * from './b'"#,
    HashMap::from([("./b".to_string(), Either::Right(file_b))]),
  );
  let members = posttype_members(&db, file_a);
  assert_contains_symbol(&members, "t1");
  assert_contains_symbol(&members, "t2");
}

#[test]
fn schema_merging_across_files() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let users_db = dbml(
    &db,
    prelude,
    "users.dbml",
    r#"auth.type UserRole {}"#,
    HashMap::new(),
  );
  let posts_db = dbml(
    &db,
    prelude,
    "posts.dbml",
    r#"auth.type PostStatus {}"#,
    HashMap::new(),
  );
  let main = dbml(
    &db,
    prelude,
    "main.dbml",
    r#"
use * from './users'
use * from './posts'
"#,
    HashMap::from([
      ("./users".to_string(), Either::Right(users_db)),
      ("./posts".to_string(), Either::Right(posts_db)),
    ]),
  );
  let members = pretype_members(&db, main);
  assert_contains_symbol(&members, "auth");
  let auth = get_first_symbol(&members, "auth");
  let auth_ns = pretype_get_symbol_namespace(&db, auth);
  assert_contains_symbol(auth_ns, "UserRole");
  assert_contains_symbol(auth_ns, "PostStatus");
}

#[test]
fn wildcard_import_includes_elements() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let dep = dbml(
    &db,
    prelude,
    "dep.dbml",
    r#"Table users { id integer [pk] }"#,
    HashMap::new(),
  );
  let main = dbml(
    &db,
    prelude,
    "main.dbml",
    r#"use * from './dep'"#,
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = posttype_members(&db, main);
  assert_contains_symbol(&members, "users");
}

#[test]
fn selective_import_of_element() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let dep = dbml(
    &db,
    prelude,
    "dep.dbml",
    r#"
Table users { id integer [pk] }
Table orders { id integer [pk] }
"#,
    HashMap::new(),
  );
  let main = dbml(
    &db,
    prelude,
    "main.dbml",
    r#"use { Table users } from './dep'"#,
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = posttype_members(&db, main);
  assert_contains_symbol(&members, "users");
  assert_missing_symbol(&members, "orders");
}

#[test]
fn use_and_reuse_same_symbol() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file_b = dbml(
    &db,
    prelude,
    "b.dbml",
    r#"Table t { id integer [pk] }"#,
    HashMap::new(),
  );
  let file_a = dbml(
    &db,
    prelude,
    "a.dbml",
    r#"
use { Table t } from './b'
reuse { Table t } from './b'
"#,
    HashMap::from([("./b".to_string(), Either::Right(file_b))]),
  );
  let file_c = dbml(
    &db,
    prelude,
    "c.dbml",
    r#"use { Table t } from './a'"#,
    HashMap::from([("./a".to_string(), Either::Right(file_a))]),
  );
  let members = posttype_members(&db, file_c);
  assert_contains_symbol(&members, "t");
}

#[test]
fn reuse_alias_is_transitive_but_use_original_is_not() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file_b = dbml(
    &db,
    prelude,
    "b.dbml",
    r#"Table users { id integer [pk] }"#,
    HashMap::new(),
  );
  let file_a = dbml(
    &db,
    prelude,
    "a.dbml",
    r#"
use { Table users } from './b'
reuse { Table users as u } from './b'
"#,
    HashMap::from([("./b".to_string(), Either::Right(file_b))]),
  );
  let file_c = dbml(
    &db,
    prelude,
    "c.dbml",
    r#"use * from './a'"#,
    HashMap::from([("./a".to_string(), Either::Right(file_a))]),
  );
  let members = posttype_members(&db, file_c);
  assert_contains_symbol(&members, "u");
  assert_missing_symbol(&members, "users");
}

#[test]
fn same_symbol_from_two_paths() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let base = dbml(
    &db,
    prelude,
    "base.dbml",
    r#"Table users { id integer [pk] }"#,
    HashMap::new(),
  );
  let alt = dbml(
    &db,
    prelude,
    "alt.dbml",
    r#"reuse * from './base'"#,
    HashMap::from([("./base".to_string(), Either::Right(base))]),
  );
  let main = dbml(
    &db,
    prelude,
    "main.dbml",
    r#"
use * from './base'
use * from './alt'
"#,
    HashMap::from([
      ("./base".to_string(), Either::Right(base)),
      ("./alt".to_string(), Either::Right(alt)),
    ]),
  );
  let members = posttype_members(&db, main);
  assert_contains_symbol(&members, "users");
}

#[test]
fn enum_import_across_files() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let dep = dbml(
    &db,
    prelude,
    "dep.dbml",
    r#"
Enum job_status {
  pending
  running
  done
}
"#,
    HashMap::new(),
  );
  let main = dbml(
    &db,
    prelude,
    "main.dbml",
    r#"
use * from './dep'
Table jobs { id integer [pk] }
"#,
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = posttype_members(&db, main);
  assert_contains_symbol(&members, "job_status");
  assert_contains_symbol(&members, "jobs");
}

#[test]
fn unresolved_import_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let dep = dbml(&db, prelude, "dep.dbml", r#"type Foo {}"#, HashMap::new());
  let main = dbml(
    &db,
    prelude,
    "main.dbml",
    r#"use { type Nonexistent } from './dep'"#,
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let diagnostics = pretype_get_file_symbols::accumulated::<Diagnostics>(&db, main, false);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::UnresolvedImport { name, .. } if name == "Nonexistent"
  )));
}
