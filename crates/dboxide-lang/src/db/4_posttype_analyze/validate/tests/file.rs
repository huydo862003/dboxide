use super::*;

#[test]
fn file_valid_element_body_no_diagnostics() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Project [element] {
  name declaration[Project] [label, unique]
  note string? [attribute]
}

Project myproject {
  note: 'A test project'
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn file_duplicate_unique_label_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
Table users {}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::DuplicateElementLabel { name, .. } if name == "users"
  )));
}
