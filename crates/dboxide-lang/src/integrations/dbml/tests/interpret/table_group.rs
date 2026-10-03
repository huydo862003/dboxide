use super::*;

#[test]
fn tablegroup_name_captured() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
TableGroup core {
  users
}
"#,
  );
  let group = find_by_name(&db, &elements, "core").expect("expected tablegroup");
  assert_eq!(
    get_field_string(&db, group, "name").as_deref(),
    Some("core")
  );
}

#[test]
fn tablegroup_note_setting_captured() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
TableGroup core [note: 'Core tables'] {
  users
}
"#,
  );
  let group = find_by_name(&db, &elements, "core").expect("expected tablegroup");
  assert_eq!(
    get_field_string(&db, group, "note").as_deref(),
    Some("Core tables"),
  );
}

#[test]
fn tablegroup_color_setting_captured() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
TableGroup core [color: '#ff9900'] {
  users
}
"#,
  );
  let group = find_by_name(&db, &elements, "core").expect("expected tablegroup");
  assert_eq!(
    get_field_string(&db, group, "color").as_deref(),
    Some("#ff9900"),
  );
}

#[test]
fn multiple_tablegroups_interpreted() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table orders { id integer [pk] }
TableGroup user_group { users }
TableGroup order_group { orders }
"#,
  );
  assert_eq!(count_by_typ(&db, &elements, "TableGroup"), 2);
  assert!(find_by_name(&db, &elements, "user_group").is_some());
  assert!(find_by_name(&db, &elements, "order_group").is_some());
}

#[test]
fn tablegroup_tables_field_populated() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let elements = interpret(
    &db,
    prelude,
    r#"
Table users { id integer [pk] }
Table posts { id integer [pk] }
TableGroup social {
  users
  posts
}
"#,
  );
  let group = find_by_name(&db, &elements, "social").expect("expected social tablegroup");
  assert!(
    has_field(&db, group, "tables"),
    "expected TableGroup to have 'tables' field, got fields: {:?}",
    group.fields(&db).keys().collect::<Vec<_>>(),
  );
  let tables = field_list(&db, group, "tables");
  assert_eq!(
    tables.len(),
    2,
    "expected 2 table entries in 'tables' field"
  );
}
