use super::*;

#[test]
fn project_instance_in_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Project myapp {
  database_type: 'PostgreSQL'
  note: 'My app'
}
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "myapp");
}
