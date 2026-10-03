use crate::SymbolCategoryKind;

use super::*;

fn find_ref_endpoints<'db>(
  db: &'db DboxideDatabase,
  elements: &[ElementObj<'db>],
) -> Vec<ElementObj<'db>> {
  elements
    .iter()
    .find(|element| match element.element_typ(db) {
      Typ::Element(element) => element.name(db).clone() == "Ref",
      _ => false,
    })
    .map(|reference| field_list(db, reference, "endpoints"))
    .unwrap_or_default()
}

fn first_ref_endpoint<'db>(
  db: &'db DboxideDatabase,
  elements: &[ElementObj<'db>],
) -> ElementObj<'db> {
  find_ref_endpoints(db, elements)
    .into_iter()
    .next()
    .expect("expected at least one ref endpoint")
}

#[test]
fn ref_delete_cascade() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [delete: cascade]
"#,
  );
  let endpoint = first_ref_endpoint(&db, &elements);
  assert_eq!(
    get_field_string(&db, &endpoint, "delete").as_deref(),
    Some("cascade")
  );
}

#[test]
fn ref_delete_restrict() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [delete: restrict]
"#,
  );
  let endpoint = first_ref_endpoint(&db, &elements);
  assert_eq!(
    get_field_string(&db, &endpoint, "delete").as_deref(),
    Some("restrict")
  );
}

#[test]
fn ref_delete_set_null() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [delete: "set null"]
"#,
  );
  let endpoint = first_ref_endpoint(&db, &elements);
  assert_eq!(
    get_field_string(&db, &endpoint, "delete").as_deref(),
    Some("set null")
  );
}

#[test]
fn ref_delete_set_default() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [delete: "set default"]
"#,
  );
  let endpoint = first_ref_endpoint(&db, &elements);
  assert_eq!(
    get_field_string(&db, &endpoint, "delete").as_deref(),
    Some("set default")
  );
}

#[test]
fn ref_delete_no_action() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [delete: "no action"]
"#,
  );
  let endpoint = first_ref_endpoint(&db, &elements);
  assert_eq!(
    get_field_string(&db, &endpoint, "delete").as_deref(),
    Some("no action")
  );
}

#[test]
fn ref_update_action() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [update: cascade]
"#,
  );
  let endpoint = first_ref_endpoint(&db, &elements);
  assert_eq!(
    get_field_string(&db, &endpoint, "update").as_deref(),
    Some("cascade")
  );
}

#[test]
fn ref_delete_and_update_together() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [delete: cascade, update: "set null"]
"#,
  );
  let endpoint = first_ref_endpoint(&db, &elements);
  assert_eq!(
    get_field_string(&db, &endpoint, "delete").as_deref(),
    Some("cascade")
  );
  assert_eq!(
    get_field_string(&db, &endpoint, "update").as_deref(),
    Some("set null")
  );
}

#[test]
fn ref_named() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref fk_user: posts.user_id > users.id
"#,
  );
  let ref_element = elements
    .iter()
    .find(|element| match element.element_typ(&db) {
      Typ::Element(element) => element.name(&db).clone() == "Ref",
      _ => false,
    })
    .expect("expected Ref element");
  assert_eq!(
    get_field_string(&db, ref_element, "name").as_deref(),
    Some("fk_user")
  );
}

#[test]
fn operator_gt_is_in_scope() {
  // Verify the `fn operator>` from prelude is importable
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(&db, prelude, "test.dbml", "", HashMap::new());
  let members = posttype_members(&db, file);
  let ops: Vec<_> = members
    .lookup_symbols(&db, ">", SymbolCategoryKind::Operator)
    .collect();
  assert!(
    !ops.is_empty(),
    "expected operator> in scope, found operators: {:?}",
    members.lookup_by_name(">").count(),
  );
}

#[test]
fn dot_access_resolves_column_element() {
  // Test that `posts.user_id` resolves to a Column ElementObj at runtime
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
"#,
    HashMap::new(),
  );
  let elements = evaluate_file(&db, file).clone();
  let posts = find_by_name(&db, &elements, "posts").expect("expected posts table");
  let columns = field_list(&db, posts, "columns");
  assert!(!columns.is_empty(), "posts should have columns");
  let column = &columns[0];
  assert_eq!(
    get_field_string(&db, column, "name").as_deref(),
    Some("user_id")
  );
}

#[test]
fn posts_namespace_contains_user_id() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
"#,
    HashMap::new(),
  );
  let members = posttype_members(&db, file);
  let posts_symbol = members
    .lookup_symbols(&db, "posts", SymbolCategoryKind::Element)
    .next()
    .expect("posts should be in scope");
  let namespace = crate::posttype_get_symbol_namespace(&db, posts_symbol);
  let user_id: Vec<_> = namespace.lookup_by_name("user_id").collect();
  assert!(
    !user_id.is_empty(),
    "posts namespace should contain user_id"
  );
}

#[test]
fn posts_table_is_in_posttype_scope() {
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
  let posts: Vec<_> = members.lookup_by_name("posts").collect();
  assert!(!posts.is_empty(), "posts should be in posttype scope");
}

#[test]
fn ref_endpoint_has_from_cardinality_to() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id
"#,
  );
  let endpoint = first_ref_endpoint(&db, &elements);
  assert!(
    has_field(&db, &endpoint, "from"),
    "expected 'from' field on RefBinary, got: {:?}",
    endpoint.fields(&db).keys().collect::<Vec<_>>(),
  );
  // Check all fields
  let all_keys: Vec<_> = endpoint.fields(&db).keys().cloned().collect();
  assert!(
    has_field(&db, &endpoint, "cardinality"),
    "expected 'cardinality' field, got: {all_keys:?}",
  );
  assert_eq!(
    get_field_string(&db, &endpoint, "cardinality").as_deref(),
    Some(">")
  );
  assert!(
    has_field(&db, &endpoint, "to"),
    "expected 'to' field on RefBinary"
  );
}

#[test]
fn ref_all_operators_produce_endpoints() {
  for op in [">", "<", "-", "<>"] {
    let db = DboxideDatabase::default();
    let prelude = setup(&db);
    let elements = interpret(
      &db,
      prelude,
      &format!(
        r#"
Table users {{ id integer [pk] }}
Table posts {{ user_id integer }}
Ref: posts.user_id {op} users.id
"#,
      ),
    );
    let endpoints = find_ref_endpoints(&db, &elements);
    assert_eq!(
      endpoints.len(),
      1,
      "expected 1 endpoint for operator '{op}'",
    );
  }
}

#[test]
fn ref_block_form() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref fk {
  posts.user_id > users.id
}
"#,
  );
  let ref_element = elements
    .iter()
    .find(|element| match element.element_typ(&db) {
      Typ::Element(element) => element.name(&db).clone() == "Ref",
      _ => false,
    })
    .expect("expected Ref element");
  assert_eq!(
    get_field_string(&db, ref_element, "name").as_deref(),
    Some("fk")
  );
  let endpoints = field_list(&db, ref_element, "endpoints");
  assert_eq!(endpoints.len(), 1);
}

#[test]
fn ref_multiple_endpoints_in_block() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Table comments { user_id integer }
Ref user_refs {
  posts.user_id > users.id
  comments.user_id > users.id
}
"#,
  );
  let ref_element = elements
    .iter()
    .find(|element| match element.element_typ(&db) {
      Typ::Element(element) => element.name(&db).clone() == "Ref",
      _ => false,
    })
    .expect("expected Ref element");
  let endpoints = field_list(&db, ref_element, "endpoints");
  assert_eq!(endpoints.len(), 2);
}
