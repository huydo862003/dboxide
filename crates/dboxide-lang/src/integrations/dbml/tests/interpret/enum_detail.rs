use super::*;

#[test]
fn enum_with_schema() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Enum auth.role {
  admin
  user
}
"#,
  );
  let element = find_by_name(&db, &elements, "role").expect("expected role enum");
  assert_eq!(
    get_field_string(&db, element, "schema").as_deref(),
    Some("auth")
  );
}

#[test]
fn enum_variant_note() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Enum status {
  pending [note: 'Waiting for review']
  active
}
"#,
  );
  let element = find_by_name(&db, &elements, "status").expect("expected status enum");
  let variants = field_list(&db, element, "variants");
  assert_eq!(variants.len(), 2);
  // First variant should have a note
  let pending = variants
    .iter()
    .find(|value| get_field_string(&db, value, "name").as_deref() == Some("pending"))
    .expect("expected pending variant");
  assert_eq!(
    get_field_string(&db, pending, "note").as_deref(),
    Some("Waiting for review"),
  );
}

#[test]
fn enum_variant_order_preserved() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Enum priority {
  low
  medium
  high
  critical
}
"#,
  );
  let element = find_by_name(&db, &elements, "priority").expect("expected priority enum");
  let variants = field_list(&db, element, "variants");
  assert_eq!(variants.len(), 4);
  assert_eq!(
    get_field_string(&db, &variants[0], "name").as_deref(),
    Some("low")
  );
  assert_eq!(
    get_field_string(&db, &variants[1], "name").as_deref(),
    Some("medium")
  );
  assert_eq!(
    get_field_string(&db, &variants[2], "name").as_deref(),
    Some("high")
  );
  assert_eq!(
    get_field_string(&db, &variants[3], "name").as_deref(),
    Some("critical")
  );
}
