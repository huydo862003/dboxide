use super::*;

#[test]
fn tablepartial_not_in_pretype_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"TablePartial timestamped { note: 'timestamps' }"#,
    HashMap::new(),
  );
  let members = pretype_members(&db, file);
  assert!(
    members.lookup_by_name("timestamped").next().is_none(),
    "tablepartial instance must not appear at pretype",
  );
}

#[test]
fn tablepartial_in_posttype_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"TablePartial timestamped { note: 'timestamps' }"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "timestamped");
  assert!(matches!(
    get_first_symbol(&members, "timestamped").kind(&db),
    SymbolKind::UserElement(..)
  ));
}

#[test]
fn multiple_tablepartials_in_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
TablePartial timestamped {}
TablePartial soft_delete {}
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "timestamped");
  assert_contains_symbol(&members, "soft_delete");
}
