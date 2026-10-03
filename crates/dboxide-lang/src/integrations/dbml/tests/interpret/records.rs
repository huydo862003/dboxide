use super::*;

fn find_child_by_type<'db>(
  db: &'db DboxideDatabase,
  element: &ElementObj<'db>,
  typ_name: &str,
) -> Option<ElementObj<'db>> {
  let children = field_list(db, element, "records");
  children
    .into_iter()
    .find(|column| match column.element_typ(db) {
      Typ::Element(element) => element.name(db).clone() == typ_name,
      _ => false,
    })
}

#[test]
fn records_nested_in_table_appears_in_children() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  Records {
    'Alice'
  }
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let records = find_child_by_type(&db, table, "Records");
  assert!(
    records.is_some(),
    "expected Records sub-element in @children"
  );
}

#[test]
fn records_example_setting_is_true() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  Records [example] {
    'Alice'
  }
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let records = find_child_by_type(&db, table, "Records").expect("expected Records");
  assert_eq!(
    extract_field_bool(&db, &records, "example"),
    Some(true),
    "expected example=true on Records with [example] setting",
  );
}

#[test]
fn records_without_example_has_no_example_field() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  Records {
    'Alice'
  }
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let records = find_child_by_type(&db, table, "Records").expect("expected Records");
  assert!(
    !has_field(&db, &records, "example"),
    "expected no example field when [example] setting is absent",
  );
}

#[test]
fn records_rows_are_captured_as_field_lines() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  Records {
    1, 'Alice'
    2, 'Bob'
  }
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let records = find_child_by_type(&db, table, "Records").expect("expected Records");
  let rows = field_list(&db, &records, "rows");
  assert_eq!(rows.len(), 2, "expected 2 row field lines");
}
