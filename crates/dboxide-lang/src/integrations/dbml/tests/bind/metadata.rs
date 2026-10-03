use super::*;

#[test]
fn metadata_not_in_pretype_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"Metadata my_target {}"#,
    HashMap::new(),
  );
  let members = pretype_members(&db, file);
  assert!(
    members.lookup_by_name("my_target").next().is_none(),
    "metadata instance must not appear at pretype",
  );
}

#[test]
fn metadata_target_label_in_posttype_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"Metadata my_target {}"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "my_target");
}

#[test]
fn metadata_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"Metadata my_target {}"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}
