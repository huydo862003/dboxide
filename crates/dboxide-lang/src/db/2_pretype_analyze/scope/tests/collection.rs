use super::*;

#[test]
fn collection_typ_declaration_appears_in_scope() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "type MyType {
  field string
}",
    HashMap::new(),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "MyType");
  assert!(matches!(
    get_first_symbol(&members, "MyType").kind(&db),
    SymbolKind::UserTyp(..)
  ));
}

#[test]
fn collection_equality_declaration_appears_in_scope() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "type Alias = string | null",
    HashMap::new(),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "Alias");
  assert!(matches!(
    get_first_symbol(&members, "Alias").kind(&db),
    SymbolKind::UserTyp(..)
  ));
}

#[test]
fn collection_elements_not_collected_in_pretype() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "Table users {
  id integer
}
Table posts {
  id integer
}",
    HashMap::new(),
  );
  let members = get_file_scope_members(&db, file);
  assert_missing_symbol(&members, "users");
  assert_missing_symbol(&members, "posts");
}

#[test]
fn collection_element_aliases_not_collected_in_pretype() {
  let db = DboxideDatabase::default();
  let file = create_file(&db, "test.dbml", "Table users as u {}", HashMap::new());
  let members = get_file_scope_members(&db, file);
  assert_missing_symbol(&members, "users");
  assert_missing_symbol(&members, "u");
}

#[test]
fn collection_schema_discovered_from_elements() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "auth.Table users {
  id integer
}",
    HashMap::new(),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "auth");
  assert_contains_symbol(&members, "public");
  assert!(matches!(
    get_first_symbol(&members, "auth").kind(&db),
    SymbolKind::UserSchema(..)
  ));
  assert_missing_symbol(&members, "users");
}
