use super::*;

#[test]
fn ref_all_operators_in_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref many_to_one: posts.user_id > users.id
Ref one_to_many: users.id < posts.user_id
Ref one_to_one: posts.user_id - users.id
Ref many_to_many: posts.user_id <> users.id
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "many_to_one");
  assert_contains_symbol(&members, "one_to_many");
  assert_contains_symbol(&members, "one_to_one");
  assert_contains_symbol(&members, "many_to_many");
}

#[test]
fn ref_with_delete_cascade_in_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref fk: posts.user_id > users.id [delete: cascade, update: set null]
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "fk");
}

#[test]
fn ref_named_in_scope() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref user_posts: posts.user_id > users.id
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "user_posts");
}

#[test]
fn ref_anonymous_produces_anonymous_element_symbol() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "posts");
  assert!(
    members
      .iter()
      .any(|(_, symbol)| matches!(symbol.kind(&db), SymbolKind::AnonymousElement(..))),
    "expected anonymous element symbol for unlabeled Ref"
  );
}

#[test]
fn ref_anonymous_imported_via_wildcard() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let dep = dbml(
    &db,
    prelude,
    "dep.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id
"#,
    HashMap::new(),
  );
  let main = dbml(
    &db,
    prelude,
    "main.dbml",
    r#"use * from './dep'"#,
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = posttype_members(&db, main);
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "posts");
  assert!(
    members
      .iter()
      .any(|(_, symbold)| matches!(symbold.kind(&db), SymbolKind::Use(inner, ..) if matches!(inner.kind(&db), SymbolKind::AnonymousElement(..)))),
    "expected anonymous Ref to be imported via wildcard"
  );
}
