use super::*;

#[test]
fn note_element_is_created() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Note {
  'This is a standalone note'
}
"#,
  );
  assert_eq!(count_by_typ(&db, &elements, "Note"), 1);
  let Typ::Element(element) = elements[0].element_typ(&db) else {
    panic!("expected Element typ");
  };
  assert_eq!(element.name(&db), "Note");
}

#[test]
fn note_inline_element_is_created() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Note: 'This is an inline note'
"#,
  );
  assert_eq!(count_by_typ(&db, &elements, "Note"), 1);
  let Typ::Element(element) = elements[0].element_typ(&db) else {
    panic!("expected Element typ");
  };
  assert_eq!(element.name(&db), "Note");
}

#[test]
fn sticky_note_name_captured() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
StickyNote readme {
  'Documentation here'
}
"#,
  );
  let note = find_by_name(&db, &elements, "readme").expect("expected sticky note");
  assert_eq!(
    get_field_string(&db, note, "name").as_deref(),
    Some("readme"),
  );
}

#[test]
fn sticky_note_color_setting_captured() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
StickyNote readme [color: '#336699'] {
  'Documentation'
}
"#,
  );
  let note = find_by_name(&db, &elements, "readme").expect("expected sticky note");
  assert_eq!(
    get_field_string(&db, note, "color").as_deref(),
    Some("#336699"),
  );
}

#[test]
fn sticky_note_color_triple_quoted() {
  // Verifies that triple-quoted strings (TqStringExpr) evaluate to string values in setting position
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
StickyNote readme [color: '''blue'''] {
  'Content'
}
"#,
  );
  let note = find_by_name(&db, &elements, "readme").expect("expected sticky note");
  assert_eq!(
    get_field_string(&db, note, "color").as_deref(),
    Some("blue"),
  );
}

#[test]
fn sticky_note_color_double_quoted_string() {
  // Verifies that double-quoted strings (DqStringExpr) fall back to string literal
  // when the value is not a resolvable symbol name
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
StickyNote readme [color: "blue"] {
  'Content'
}
"#,
  );
  let note = find_by_name(&db, &elements, "readme").expect("expected sticky note");
  assert_eq!(
    get_field_string(&db, note, "color").as_deref(),
    Some("blue"),
  );
}

#[test]
fn multiple_notes_are_created() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Note { 'first' }
Note { 'second' }
StickyNote pinned { 'third' }
"#,
  );
  assert_eq!(count_by_typ(&db, &elements, "Note"), 2);
  assert_eq!(count_by_typ(&db, &elements, "StickyNote"), 1);
}
