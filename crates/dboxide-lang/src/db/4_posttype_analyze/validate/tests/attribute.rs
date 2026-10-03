use super::*;

#[test]
fn attribute_duplicate_unique_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  pk bool [attribute, unique]
}

Table users {
  pk: true
  pk: true
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::DuplicateAttribute { name, .. } if name == "pk"
  )));
}

#[test]
fn attribute_duplicate_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  pk bool [attribute]
}

Table users {
  pk: true
  pk: true
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(
    diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::DuplicateAttribute { name, .. } if name == "pk"
    )),
    "expected DuplicateAttribute diagnostic - attribute is always unique"
  );
}

#[test]
fn attribute_unknown_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Project [element] {
  name declaration[Project] [label, unique]
}

Project myproject {
  nonexistent: 'value'
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::UnknownAttribute { name, typ_name, .. }
      if name == "nonexistent" && typ_name == "Project"
  )));
}

#[test]
fn attribute_rest_non_map_typ_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Project [element] {
  name declaration[Project] [label, unique]
  [extra] string
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(
    diagnostics
      .iter()
      .any(|diagnostic| matches!(&diagnostic.0.kind, DiagnosticKind::RestAttributeMustBeMap))
  );
}

#[test]
fn attribute_rest_map_typ_no_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Project [element] {
  name declaration[Project] [label, unique]
  [extra] Map
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(
    !diagnostics
      .iter()
      .any(|diagnostic| matches!(&diagnostic.0.kind, DiagnosticKind::RestAttributeMustBeMap)),
    "unexpected RestAttributeMustBeMap diagnostic"
  );
}
