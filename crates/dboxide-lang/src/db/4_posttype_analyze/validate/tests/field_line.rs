use super::*;

#[test]
fn field_line_too_few_args_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  type Column [field] {
    col_name string [arg: 0]
    col_type string [arg: 1]
  }
}

Table users {
  id
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::TooFewFieldLineArgs { typ_name, min_expected: 2, found: 1 }
      if typ_name == "Column"
  )));
}

#[test]
fn field_line_too_many_args_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  type Column [field] {
    col_name string [arg: 0]
  }
}

Table users {
  id extra_arg
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::TooManyFieldLineArgs { typ_name, max_expected: 1, found: 2 }
      if typ_name == "Column"
  )));
}

#[test]
fn field_line_correct_arg_count_no_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  type Column [field] {
    col_name string [arg: 0]
    col_type string [arg: 1]
  }
}

Table users {
  id integer
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(
    !diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::TooFewFieldLineArgs { .. } | DiagnosticKind::TooManyFieldLineArgs { .. }
    )),
    "unexpected arg count diagnostic"
  );
}

#[test]
fn field_line_variadic_accepts_any_count() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Index [element] {
  name declaration[Index] [label, unique]
  type IndexField [field] {
    col_name string [arg: 0]
    extra string [arg: 1..]
  }
}

Index pk_idx {
  IndexField id
  IndexField id email name
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(
    !diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::TooFewFieldLineArgs { .. } | DiagnosticKind::TooManyFieldLineArgs { .. }
    )),
    "variadic arg should accept any count >= non_variadic_count"
  );
}
