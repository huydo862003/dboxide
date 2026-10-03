use super::*;

#[test]
fn enum_instance_in_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Enum status {
  active
  inactive
}
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "status");
}

#[test]
fn enum_with_schema() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Enum auth.role {
  admin
  user
}
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "auth");
}

#[test]
fn duplicate_enum_field_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Enum status {
  active
  inactive
  active
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::DuplicateFieldDeclaration { name, .. } if name == "active"
    )
  });
}

#[test]
fn enum_fields_isolated_to_enum_scope() {
  // Variant names in different enums do not conflict
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Enum color { red, green, blue }
Enum priority { low, medium, high }
"#,
    HashMap::new(),
  );
  let diagnostics = posttype_get_file_symbols::accumulated::<Diagnostics>(&db, file, false);
  assert!(
    !diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::DuplicateElementLabel { .. }
    )),
    "variant names in different enums must not conflict"
  );
}

#[test]
fn multiple_enums() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Enum status { active, inactive }
Enum priority { low, medium, high }
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "status");
  assert_contains_symbol(&members, "priority");
}
