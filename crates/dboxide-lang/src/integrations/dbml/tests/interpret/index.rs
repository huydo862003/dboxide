use super::*;

fn table_indexes<'db>(
  db: &'db DboxideDatabase,
  elements: &[ElementObj<'db>],
  table_name: &str,
) -> Vec<ElementObj<'db>> {
  let table = find_by_name(db, elements, table_name).expect("expected table");
  field_list(db, table, "indexes")
}

fn index_columns<'db>(db: &'db DboxideDatabase, index: &ElementObj<'db>) -> Vec<ElementObj<'db>> {
  field_list(db, index, "columns")
}

#[test]
fn simple_index() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar
  Index { email }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  assert_eq!(indexes.len(), 1);
}

#[test]
fn unique_index() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar
  Index { email [unique] }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  let cols = index_columns(&db, &indexes[0]);
  assert_eq!(cols.len(), 1);
  assert_eq!(extract_field_bool(&db, &cols[0], "unique"), Some(true));
}

#[test]
fn named_index() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar
  Index { email [name: 'idx_email'] }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  let cols = index_columns(&db, &indexes[0]);
  assert_eq!(
    get_field_string(&db, &cols[0], "name").as_deref(),
    Some("idx_email"),
  );
}

#[test]
fn pk_index() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer
  Index { id [pk] }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  let cols = index_columns(&db, &indexes[0]);
  assert_eq!(extract_field_bool(&db, &cols[0], "pk"), Some(true));
}

#[test]
fn index_type_btree() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar
  Index { email [type: btree] }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  let cols = index_columns(&db, &indexes[0]);
  assert_eq!(
    get_field_string(&db, &cols[0], "type").as_deref(),
    Some("btree")
  );
}

#[test]
fn index_type_hash() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar
  Index { email [type: hash] }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  let cols = index_columns(&db, &indexes[0]);
  assert_eq!(
    get_field_string(&db, &cols[0], "type").as_deref(),
    Some("hash")
  );
}

#[test]
fn index_note() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar
  Index { email [note: 'Fast email lookup'] }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  let cols = index_columns(&db, &indexes[0]);
  assert_eq!(
    get_field_string(&db, &cols[0], "note").as_deref(),
    Some("Fast email lookup"),
  );
}

#[test]
fn index_all_settings() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar
  Index { email [unique, name: 'idx_email', type: hash, note: 'Email index'] }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  let cols = index_columns(&db, &indexes[0]);
  assert_eq!(extract_field_bool(&db, &cols[0], "unique"), Some(true));
  assert_eq!(
    get_field_string(&db, &cols[0], "name").as_deref(),
    Some("idx_email")
  );
  assert_eq!(
    get_field_string(&db, &cols[0], "type").as_deref(),
    Some("hash")
  );
  assert_eq!(
    get_field_string(&db, &cols[0], "note").as_deref(),
    Some("Email index")
  );
}

#[test]
fn multiple_indexes() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar
  name varchar
  Index { email [unique] }
  Index { name }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  assert_eq!(indexes.len(), 2);
}

#[test]
fn composite_index() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  first_name varchar
  last_name varchar
  Index {
    first_name
    last_name [unique]
  }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  assert_eq!(indexes.len(), 1);
  let cols = index_columns(&db, &indexes[0]);
  assert_eq!(cols.len(), 2, "expected 2 columns in composite index");
}

#[test]
fn expression_index() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users {
  id integer [pk]
  email varchar
  Index { `lower(email)` }
}
"#,
  );
  let indexes = table_indexes(&db, &elements, "users");
  assert_eq!(indexes.len(), 1);
}
