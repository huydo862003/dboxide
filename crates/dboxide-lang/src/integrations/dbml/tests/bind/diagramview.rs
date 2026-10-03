use super::*;

#[test]
fn diagramview_not_in_pretype_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"DiagramView main {}"#,
    HashMap::new(),
  );
  let members = pretype_members(&db, file);
  assert!(
    members.lookup_by_name("main").next().is_none(),
    "diagramview instance must not appear at pretype",
  );
}

#[test]
fn diagramview_in_posttype_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"DiagramView main {}"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "main");
  assert!(matches!(
    get_first_symbol(&members, "main").kind(&db),
    SymbolKind::UserElement(..)
  ));
}

#[test]
fn diagramview_with_sub_elements_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
DiagramView main {
  Tables {
    users
  }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn multiple_diagramviews_in_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
DiagramView overview {}
DiagramView detail {}
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "overview");
  assert_contains_symbol(&members, "detail");
}
