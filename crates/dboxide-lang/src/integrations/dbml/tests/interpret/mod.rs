mod column;
mod dep;
mod enum_detail;
mod index;
mod note;
mod records;
mod ref_detail;
mod ref_element;
mod table_group;

use super::*;

use crate::{ElementObj, Obj, Typ, evaluate_file};

fn interpret<'db>(db: &'db DboxideDatabase, prelude: File, content: &str) -> Vec<ElementObj<'db>> {
  let file = dbml(db, prelude, "test.dbml", content, HashMap::new());
  evaluate_file(db, file).clone()
}

fn find_by_name<'a, 'db>(
  db: &'db DboxideDatabase,
  elements: &'a [ElementObj<'db>],
  name: &str,
) -> Option<&'a ElementObj<'db>> {
  elements.iter().find(|element| {
    element
      .fields(db)
      .get("name")
      .and_then(|obj| match obj {
        Obj::String(string) => Some(string.value(db) == name),
        _ => None,
      })
      .unwrap_or(false)
  })
}

fn count_by_typ<'db>(
  db: &'db DboxideDatabase,
  elements: &[ElementObj<'db>],
  typ_name: &str,
) -> usize {
  elements
    .iter()
    .filter(|element| match element.element_typ(db) { Typ::Element(element) => element.name(db).clone(), _ => String::new() } == typ_name)
    .count()
}

fn has_field(db: &DboxideDatabase, element: &ElementObj, key: &str) -> bool {
  element.fields(db).contains_key(key)
}

fn get_field_string<'db>(
  db: &'db DboxideDatabase,
  element: &ElementObj<'db>,
  key: &str,
) -> Option<String> {
  element.fields(db).get(key).and_then(|obj| match obj {
    Obj::String(s) => Some(s.value(db).to_string()),
    _ => None,
  })
}

fn extract_field_bool<'db>(
  db: &'db DboxideDatabase,
  element: &ElementObj<'db>,
  key: &str,
) -> Option<bool> {
  element.fields(db).get(key).and_then(|obj| match obj {
    Obj::Bool(bool) => Some(bool.value(db)),
    _ => None,
  })
}

fn get_field_color<'db>(
  db: &'db DboxideDatabase,
  element: &ElementObj<'db>,
  key: &str,
) -> Option<(u8, u8, u8)> {
  element.fields(db).get(key).and_then(|obj| match obj {
    Obj::Color(color) => Some((color.r(db), color.g(db), color.b(db))),
    _ => None,
  })
}

fn field_list<'db>(
  db: &'db DboxideDatabase,
  element: &ElementObj<'db>,
  key: &str,
) -> Vec<ElementObj<'db>> {
  element
    .fields(db)
    .get(key)
    .and_then(|obj| match obj {
      Obj::List(list) => Some(
        list
          .items(db)
          .iter()
          .filter_map(|item| match item {
            Obj::Element(element) => Some(*element),
            _ => None,
          })
          .collect(),
      ),
      _ => None,
    })
    .unwrap_or_default()
}

#[test]
fn table_double_quoted_identifier_name() {
  // Verifies that double-quoted strings (DqStringExpr) work as element names (identifiers with spaces)
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table "my users" {
  id integer [pk]
}
"#,
  );
  assert_eq!(count_by_typ(&db, &elements, "Table"), 1);
  assert!(
    find_by_name(&db, &elements, "my users").is_some(),
    "expected table named 'my users'",
  );
}

#[test]
fn interpret_simple_table() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  name string
}
"#,
  );
  assert_eq!(count_by_typ(&db, &elements, "Table"), 1);
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  assert_eq!(
    get_field_string(&db, table, "name").as_deref(),
    Some("users")
  );
}

#[test]
fn interpret_table_with_alias() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users as u {
  id integer [pk]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  assert_eq!(get_field_string(&db, table, "alias").as_deref(), Some("u"));
}

#[test]
fn interpret_table_with_schema() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table auth.users {
  id integer [pk]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  assert_eq!(
    get_field_string(&db, table, "schema").as_deref(),
    Some("auth")
  );
}

#[test]
fn interpret_table_column_settings() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk, increment]
  email string [unique, not null]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  assert!(has_field(&db, table, "columns"));
}

#[test]
fn interpret_table_note_attribute() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  note: 'Main users table'
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  assert_eq!(
    get_field_string(&db, table, "note").as_deref(),
    Some("Main users table")
  );
}

#[test]
fn interpret_table_header_setting() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users [headercolor: '#ff0000'] {
  id integer [pk]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  assert_eq!(
    get_field_string(&db, table, "headercolor").as_deref(),
    Some("#ff0000")
  );
}

#[test]
fn interpret_enum_with_variants() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Enum status {
  pending
  active
  inactive
}
"#,
  );
  assert_eq!(count_by_typ(&db, &elements, "Enum"), 1);
  let e = find_by_name(&db, &elements, "status").expect("expected status enum");
  // Enum variants are field lines, stored as @fields list
  let field_lines = field_list(&db, e, "variants");
  assert_eq!(field_lines.len(), 3, "expected 3 enum variants");
}

#[test]
fn interpret_project() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Project myapp {
  database_type: 'PostgreSQL'
  note: 'My application database'
}
"#,
  );
  let project = find_by_name(&db, &elements, "myapp").expect("expected project");
  assert_eq!(
    get_field_string(&db, project, "database_type").as_deref(),
    Some("PostgreSQL")
  );
  assert_eq!(
    get_field_string(&db, project, "note").as_deref(),
    Some("My application database")
  );
}

#[test]
fn interpret_sticky_note() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
StickyNote readme {
  'This is the readme'
}
"#,
  );
  let note = find_by_name(&db, &elements, "readme").expect("expected sticky note");
  let Typ::Element(element) = note.element_typ(&db) else {
    panic!("expected Element typ")
  };
  assert_eq!(element.name(&db), "StickyNote");
}

#[test]
fn interpret_multiple_tables() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email string [unique]
}

Table posts {
  id integer [pk]
  title string
}

Table comments {
  id integer [pk]
  body string
}
"#,
  );
  assert_eq!(count_by_typ(&db, &elements, "Table"), 3);
  assert!(find_by_name(&db, &elements, "users").is_some());
  assert!(find_by_name(&db, &elements, "posts").is_some());
  assert!(find_by_name(&db, &elements, "comments").is_some());
}

#[test]
fn interpret_full_program() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Project ecommerce {
  database_type: 'PostgreSQL'
}

Enum order_status {
  pending
  processing
  shipped
}

Table users {
  id integer [pk, increment]
  email string [unique, not null]
}

Table orders {
  id integer [pk]
  user_id integer [not null]
}

StickyNote readme {
  'E-commerce schema'
}
"#,
  );
  assert!(find_by_name(&db, &elements, "ecommerce").is_some());
  assert!(find_by_name(&db, &elements, "order_status").is_some());
  assert!(find_by_name(&db, &elements, "users").is_some());
  assert!(find_by_name(&db, &elements, "orders").is_some());
  assert!(find_by_name(&db, &elements, "readme").is_some());
}

/* Color literals */

#[test]
fn sticky_note_color_six_digit_hex() {
  // Verifies that #rrggbb color literals evaluate to ColorObj with correct r/g/b channels
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
StickyNote readme [color: #336699] {
  'Content'
}
"#,
  );
  let note = find_by_name(&db, &elements, "readme").expect("expected sticky note");
  assert_eq!(
    get_field_color(&db, note, "color"),
    Some((0x33, 0x66, 0x99)),
  );
}

#[test]
fn sticky_note_color_three_digit_hex() {
  // Verifies that #rgb shorthand expands each digit (e.g. #f0a = #ff00aa)
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
StickyNote readme [color: #f0a] {
  'Content'
}
"#,
  );
  let note = find_by_name(&db, &elements, "readme").expect("expected sticky note");
  assert_eq!(
    get_field_color(&db, note, "color"),
    Some((0xff, 0x00, 0xaa)),
  );
}
