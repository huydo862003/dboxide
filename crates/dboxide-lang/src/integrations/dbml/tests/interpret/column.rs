use super::*;

#[test]
fn column_type_is_captured() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  name varchar
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(columns.len(), 2);

  let id_column = columns
    .iter()
    .find(|column| get_field_string(&db, column, "name").as_deref() == Some("id"))
    .expect("expected id column");
  assert_eq!(
    get_field_string(&db, id_column, "type").as_deref(),
    Some("integer")
  );

  let name_column = columns
    .iter()
    .find(|column| get_field_string(&db, column, "name").as_deref() == Some("name"))
    .expect("expected name column");
  assert_eq!(
    get_field_string(&db, name_column, "type").as_deref(),
    Some("varchar")
  );
}

#[test]
fn column_type_with_args() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  name varchar(255)
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(columns.len(), 1);
  let type_value = get_field_string(&db, &columns[0], "type").expect("expected type field");
  assert!(
    type_value.contains("varchar"),
    "expected type to contain 'varchar', got '{type_value}'",
  );
}

#[test]
fn column_type_double_quoted_text_field() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id "varchar(36)" [pk]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(columns.len(), 1);
  assert_eq!(
    get_field_string(&db, &columns[0], "type").as_deref(),
    Some("varchar(36)"),
  );
}

#[test]
fn column_pk_setting() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(extract_field_bool(&db, &columns[0], "pk"), Some(true));
}

#[test]
fn column_unique_setting() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  email varchar [unique]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(extract_field_bool(&db, &columns[0], "unique"), Some(true));
}

#[test]
fn column_not_null_setting() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  name varchar [not null]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(extract_field_bool(&db, &columns[0], "not_null"), Some(true));
}

#[test]
fn column_increment_setting() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk, increment]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(
    extract_field_bool(&db, &columns[0], "increment"),
    Some(true)
  );
}

#[test]
fn column_default_string() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  status varchar [default: 'active']
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert!(has_field(&db, &columns[0], "default"));
}

#[test]
fn column_default_number() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  age integer [default: 0]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert!(has_field(&db, &columns[0], "default"));
}

#[test]
fn column_note() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  email varchar [note: 'User email address']
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(
    get_field_string(&db, &columns[0], "note").as_deref(),
    Some("User email address"),
  );
}

#[test]
fn column_all_settings_together() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk, not null, unique, increment]
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(extract_field_bool(&db, &columns[0], "pk"), Some(true));
  assert_eq!(extract_field_bool(&db, &columns[0], "not_null"), Some(true));
  assert_eq!(extract_field_bool(&db, &columns[0], "unique"), Some(true));
  assert_eq!(
    extract_field_bool(&db, &columns[0], "increment"),
    Some(true)
  );
}

#[test]
fn multiple_columns_preserved_in_order() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar [unique]
  name varchar
  age integer
}
"#,
  );
  let table = find_by_name(&db, &elements, "users").expect("expected users table");
  let columns = field_list(&db, table, "columns");
  assert_eq!(columns.len(), 4);
  assert_eq!(
    get_field_string(&db, &columns[0], "name").as_deref(),
    Some("id")
  );
  assert_eq!(
    get_field_string(&db, &columns[1], "name").as_deref(),
    Some("email")
  );
  assert_eq!(
    get_field_string(&db, &columns[2], "name").as_deref(),
    Some("name")
  );
  assert_eq!(
    get_field_string(&db, &columns[3], "name").as_deref(),
    Some("age")
  );
}
