use super::*;

#[test]
fn tablegroup_multifile_expansion() {
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
fn tablegroup_wildcard_import_expands_tables() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file_a = dbml(
    &db,
    prelude,
    "a.dbml",
    r#"
Table auth_users { id integer [pk] }
TableGroup auth_group {
  auth_users
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
  assert_contains_symbol(&members, "auth_group");
  assert_contains_symbol(&members, "auth_users");
}

#[test]
fn tablegroup_in_scope() {
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
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "social");
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "posts");
}
