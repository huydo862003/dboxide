use super::*;

#[test]
fn setting_duplicate_unique_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  note string [setting, unique]
}

Table users [note: 'first', note: 'second'] {}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::DuplicateSetting { name, .. } if name == "note"
  )));
}

#[test]
fn setting_duplicate_non_unique_no_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  note string [setting]
}

Table users [note: 'first', note: 'second'] {}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(
    !diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::DuplicateSetting { name, .. } if name == "note"
    )),
    "unexpected DuplicateSetting diagnostic for non-unique setting"
  );
}

#[test]
fn setting_unknown_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
}

Table users [badkey: red] {}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::UnknownSetting { name, .. } if name == "badkey"
  )));
}
