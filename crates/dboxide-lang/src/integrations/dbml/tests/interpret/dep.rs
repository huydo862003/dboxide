use super::*;

#[test]
fn dep_note_attribute_is_captured() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Dep staging {
  note: 'Pipeline from raw to fact'
}
"#,
  );
  let dep = find_by_name(&db, &elements, "staging").expect("expected dep");
  assert_eq!(
    get_field_string(&db, dep, "note").as_deref(),
    Some("Pipeline from raw to fact"),
  );
}

#[test]
fn dep_color_attribute_is_captured() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Dep staging {
  color: '#aabbcc'
}
"#,
  );
  let dep = find_by_name(&db, &elements, "staging").expect("expected dep");
  assert_eq!(
    get_field_string(&db, dep, "color").as_deref(),
    Some("#aabbcc")
  );
}

#[test]
fn dep_note_from_header_setting_is_captured() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Dep staging [note: 'from header'] {
}
"#,
  );
  let dep = find_by_name(&db, &elements, "staging").expect("expected dep");
  assert_eq!(
    get_field_string(&db, dep, "note").as_deref(),
    Some("from header"),
  );
}

#[test]
fn dep_anonymous_is_interpreted() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Dep {
  note: 'anonymous dep'
}
"#,
  );
  assert_eq!(count_by_typ(&db, &elements, "Dep"), 1);
  let dep = elements
    .iter()
    .find(|element| match element.element_typ(&db) {
      Typ::Element(element) => element.name(&db).clone() == "Dep",
      _ => false,
    })
    .expect("expected dep");
  assert!(!has_field(&db, dep, "name"));
  assert_eq!(
    get_field_string(&db, dep, "note").as_deref(),
    Some("anonymous dep"),
  );
}

#[test]
fn dep_body_attribute_overrides_header_setting() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Dep staging [note: 'header'] {
  note: 'body'
}
"#,
  );
  let dep = find_by_name(&db, &elements, "staging").expect("expected dep");
  assert_eq!(get_field_string(&db, dep, "note").as_deref(), Some("body"));
}
