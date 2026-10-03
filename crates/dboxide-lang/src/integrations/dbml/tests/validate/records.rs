use super::*;

#[test]
fn nested_records_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  name varchar
  Records {
    1, 'Alice'
    2, 'Bob'
  }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn nested_records_with_example_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  name varchar
  Records [example] {
    1, 'Alice'
  }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn nested_records_empty_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  Records {}
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn nested_records_various_value_types_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table data {
  id integer
  active bool
  score decimal
  Records {
    1, true, 3.14
    2, false, 0.5
  }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

/* not null */

#[test]
fn records_null_in_not_null_column_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  name string [not null]
  Records {
    1, null
  }
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::ConstraintViolation { constraint_name, .. } if constraint_name == "not_null"
    )
  });
}

#[test]
fn records_null_in_increment_column_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk, increment]
  name string
  Records {
    null, 'Alice'
  }
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::ConstraintViolation { constraint_name, .. } if constraint_name == "not_null"
    )
  });
}

#[test]
fn records_null_in_nullable_column_no_diagnostic() {
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
  Records {
    1, null
  }
}
"#,
    HashMap::new(),
  );
  assert_no_matching_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::ConstraintViolation { constraint_name, .. } if constraint_name == "not_null"
    )
  });
}

/* pk */

#[test]
fn records_duplicate_pk_emits_diagnostic() {
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
  Records {
    1, 'Alice'
    1, 'Bob'
  }
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::ConstraintViolation { constraint_name, .. } if constraint_name == "pk_unique"
    )
  });
}

#[test]
fn records_null_pk_emits_diagnostic() {
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
  Records {
    null, 'Alice'
  }
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::ConstraintViolation { constraint_name, .. } if constraint_name == "not_null"
    )
  });
}

#[test]
fn records_unique_pk_values_no_diagnostic() {
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
  Records {
    1, 'Alice'
    2, 'Bob'
    3, 'Carol'
  }
}
"#,
    HashMap::new(),
  );
  assert_no_matching_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::ConstraintViolation { constraint_name, .. } if constraint_name == "pk_unique"
    )
  });
}

/* unique */

#[test]
fn records_duplicate_unique_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  email string [unique]
  Records {
    1, 'alice@example.com'
    2, 'alice@example.com'
  }
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::ConstraintViolation { constraint_name, .. } if constraint_name == "unique_check"
    )
  });
}

#[test]
fn records_null_in_unique_column_allowed() {
  // NULL values do not conflict with each other in unique columns (SQL standard)
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  email string [unique]
  Records {
    1, null
    2, null
  }
}
"#,
    HashMap::new(),
  );
  assert_no_matching_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::ConstraintViolation { constraint_name, .. } if constraint_name == "unique_check"
    )
  });
}

/* settings validation */

#[test]
fn records_unknown_setting_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  Records [note: 'test'] {
    1
  }
}
"#,
    HashMap::new(),
  );
  assert_has_diagnostic(&db, file, |kind| {
    matches!(
      kind,
      DiagnosticKind::UnknownSetting { name, .. } if name == "note"
    )
  });
}

#[test]
fn records_example_flag_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table users {
  id integer [pk]
  Records [example] {
    1
  }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn records_negative_numbers_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table data {
  id integer
  value integer
  Records {
    1, -100
    2, -999
  }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn records_null_true_false_values_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table data {
  id integer
  active bool
  name varchar
  Records {
    1, true, null
    2, false, null
  }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}

#[test]
fn records_single_column_no_diagnostic() {
  let db = DboxideDatabase::default();
  let prelude = setup(&db);
  let file = dbml(
    &db,
    prelude,
    "test.dbml",
    r#"
Table ids {
  id integer [pk]
  Records {
    1
    2
    3
  }
}
"#,
    HashMap::new(),
  );
  assert_no_diagnostics(&db, file);
}
