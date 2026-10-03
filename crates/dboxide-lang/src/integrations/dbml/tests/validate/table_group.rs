use super::*;

#[test]
fn tablegroup_valid_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { id integer [pk] }
TableGroup social {
  users
  posts
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn tablegroup_with_settings_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
TableGroup core [note: 'Core tables', color: '#336699'] {
  users
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn tablegroup_with_schema_qualified_table_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table auth.users { id integer [pk] }
TableGroup auth_group {
  auth.users
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn tablegroup_use_expands_tables_into_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file_a = dbml(
    &db,
    prelude,
    "a.dbml",
    r#"
Table users { id integer [pk] }
Table posts { id integer [pk] }
TableGroup social {
  users
  posts
}
"#,
    HashMap::new(),
  );
  let file_b = dbml(
    &db,
    prelude,
    "b.dbml",
    r#"use { tablegroup social } from './a'"#,
    HashMap::from([("./a".to_string(), Either::Right(file_a))]),
  );
  let members = posttype_members(&db, file_b);
  assert_contains_symbol(&members, "social");
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "posts");
}

#[test]
fn use_wildcard_from_schema_with_tablegroup_brings_tables_into_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file_a = dbml(
    &db,
    prelude,
    "a.dbml",
    r#"
Table users { id integer [pk] }
Table posts { id integer [pk] }
TableGroup social {
  users
  posts
}
"#,
    HashMap::new(),
  );
  let file_b = dbml(
    &db,
    prelude,
    "b.dbml",
    r#"use * from './a'"#,
    HashMap::from([("./a".to_string(), Either::Right(file_a))]),
  );
  let members = posttype_members(&db, file_b);
  assert_contains_symbol(&members, "social");
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "posts");
}
