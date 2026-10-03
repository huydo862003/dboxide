use super::*;

#[test]
fn dep_not_in_pretype_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Dep staging {
  note: 'Pipeline'
}
"#,
    HashMap::new(),
  );
  let members = pretype_members(&db, file);
  assert!(
    members.lookup_by_name("staging").next().is_none(),
    "dep instance must not appear at pretype",
  );
}

#[test]
fn dep_named_in_posttype_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Dep staging {
  note: 'Pipeline'
}
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "staging");
  assert!(matches!(
    get_first_symbol(&members, "staging").kind(&db),
    SymbolKind::UserElement(..)
  ));
}

#[test]
fn dep_anonymous_has_no_scope_entry() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Dep {
  note: 'anonymous'
}
"#,
    HashMap::new(),
  );
  // anonymous Dep has no label, so nothing is added to scope
  assert_no_diagnostics(&db, file);
}

#[test]
fn multiple_deps_in_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Dep staging {}
Dep marts {}
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "staging");
  assert_contains_symbol(&members, "marts");
}
