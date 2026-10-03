use super::*;

#[test]
fn table_instance_in_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  name string
}
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "users");
  assert!(matches!(
    get_first_symbol(&members, "users").kind(&db),
    SymbolKind::UserElement(..)
  ));
}

#[test]
fn table_with_alias() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"Table users as u { id integer [pk] }"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "u");
}

#[test]
fn table_with_schema() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table auth.roles { id integer [pk] }
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "auth");
}

#[test]
fn multiple_tables() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { id integer [pk] }
Table comments { id integer [pk] }
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "posts");
  assert_contains_symbol(&members, "comments");
}

#[test]
fn quoted_column_name() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  "user id" integer [pk]
  "full name" string
}
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "users");
}

#[test]
fn duplicate_column_name_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  name string
  id varchar
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::DuplicateFieldDeclaration { name, .. } if name == "id"
    )
  });
}

#[test]
fn duplicate_columns_across_tables_allowed() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { id integer [pk] }
"#,
    HashMap::new(),
  );
  assert_no_matching_diagnostic(&db, file, |kind| {
    matches!(kind, DiagnosticKind::DuplicateFieldDeclaration { .. })
  });
}

#[test]
fn same_table_name_different_schemas_allowed() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table auth.users { id integer [pk] }
Table billing.users { id integer [pk] }
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "auth");
  assert_contains_symbol(&members, "billing");
}
