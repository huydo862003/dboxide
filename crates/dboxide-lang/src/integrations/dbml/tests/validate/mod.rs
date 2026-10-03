mod records;
mod ref_element;
mod table_group;

use super::*;

#[test]
fn valid_table_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk, increment]
  email string [unique, not null, note: 'User email']
  name string [null]
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn duplicate_table_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table users { id integer [pk] }
"#,
    HashMap::new(),
  );
  let diagnostics = posttype_get_file_symbols::accumulated::<Diagnostics>(&db, file, false);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::DuplicateElementLabel { name, .. } if name == "users"
  )));
}

#[test]
fn duplicate_enum_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Enum status { active }
Enum status { draft }
"#,
    HashMap::new(),
  );
  let diagnostics = posttype_get_file_symbols::accumulated::<Diagnostics>(&db, file, false);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::DuplicateElementLabel { name, .. } if name == "status"
  )));
}

#[test]
fn duplicate_project_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Project a { database_type: 'PostgreSQL' }
Project a { database_type: 'MySQL' }
"#,
    HashMap::new(),
  );
  let diagnostics = posttype_get_file_symbols::accumulated::<Diagnostics>(&db, file, false);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::DuplicateElementLabel { name, .. } if name == "a"
  )));
}

#[test]
fn duplicate_sticky_note_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
StickyNote readme { 'first' }
StickyNote readme { 'second' }
"#,
    HashMap::new(),
  );
  let diagnostics = posttype_get_file_symbols::accumulated::<Diagnostics>(&db, file, false);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::DuplicateElementLabel { name, .. } if name == "readme"
  )));
}

#[test]
fn duplicate_tablegroup_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
TableGroup core { users }
TableGroup core { users }
"#,
    HashMap::new(),
  );
  let diagnostics = posttype_get_file_symbols::accumulated::<Diagnostics>(&db, file, false);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::DuplicateElementLabel { name, .. } if name == "core"
  )));
}

#[test]
fn same_table_name_different_schemas_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table auth.users { id integer [pk] }
Table billing.users { id integer [pk] }
"#,
    HashMap::new(),
  );
  let diagnostics = posttype_get_file_symbols::accumulated::<Diagnostics>(&db, file, false);
  assert!(
    !diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::DuplicateElementLabel { .. }
    )),
    "different schemas should not produce duplicate label diagnostic"
  );
}

#[test]
fn table_with_note_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  note: 'Main users table'
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn enum_with_variant_notes_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Enum status {
  pending [note: 'Waiting']
  active
  inactive
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn project_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Project myapp {
  database_type: 'PostgreSQL'
  note: 'Main database'
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn table_with_alias_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users as u {
  id integer [pk]
  email string [unique, not null]
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn column_all_settings_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk, not null, unique, increment]
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn column_pk_not_null_combination_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk, not null]
  email string [unique, not null]
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn inline_pk_unique_flags_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table orders {
  id integer [pk]
  order_num string [unique]
  status string [null]
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn increment_without_null_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk, increment]
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn increment_with_null_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk, increment, null]
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::ConstraintViolation { constraint_name, .. } if constraint_name == "increment_not_null"
    )
  });
}

#[test]
fn tablepartial_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
TablePartial timestamped {
  note: 'common timestamp fields'
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn ref_named_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref fk_user: posts.user_id > users.id
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn ref_all_operators_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Table tags { id integer [pk] }
Table post_tags { post_id integer
  tag_id integer }
Ref: posts.user_id > users.id
Ref: users.id < posts.user_id
Ref: post_tags.post_id - posts.id
Ref: post_tags.tag_id <> tags.id
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn ref_with_settings_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users { id integer [pk] }
Table posts { user_id integer }
Ref: posts.user_id > users.id [delete: cascade, update: set null]
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn duplicate_column_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  name string
  id varchar
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::DuplicateFieldDeclaration { name, .. } if name == "id"
    )
  });
}

#[test]
fn duplicate_enum_variant_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Enum status {
  active
  inactive
  active
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::DuplicateFieldDeclaration { name, .. } if name == "active"
    )
  });
}

#[test]
fn column_with_default_numeric_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  age integer [default: 0]
  score integer [default: 100]
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn column_with_default_string_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  status string [default: 'active']
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn index_valid_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table orders {
  id integer [pk]
  user_id integer
  Index { user_id }
  Index { id user_id [unique] }
  Index { id [pk] }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn index_with_settings_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table products {
  id integer [pk]
  sku string
  Index { sku [unique, name: 'idx_sku', type: hash, note: 'SKU index'] }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn full_dbml_program_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Project ecommerce {
  database_type: 'PostgreSQL'
  note: 'E-commerce database'
}

Enum order_status {
  pending
  processing
  shipped
  delivered
}

Table users {
  id integer [pk, increment]
  email string [unique, not null]
  name string
}

Table orders {
  id integer [pk, increment]
  user_id integer [not null]
  total integer
}

StickyNote schema_note {
  'Core e-commerce schema'
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn table_inside_table_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table outer {
  id integer [pk]
  Table inner { id integer [pk] }
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(
    &db,
    file,
    |kind| matches!(kind, DiagnosticKind::UnknownSubElement { name, .. } if name == "Table"),
  );
}

#[test]
fn enum_inside_table_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  Enum status { active }
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(
    &db,
    file,
    |kind| matches!(kind, DiagnosticKind::UnknownSubElement { name, .. } if name == "Enum"),
  );
}

#[test]
fn note_inside_table_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  note: 'Valid note'
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn index_inside_table_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  Index { id }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}
