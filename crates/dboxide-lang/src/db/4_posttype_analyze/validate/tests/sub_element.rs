use super::*;

#[test]
fn sub_element_unknown_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
}

Table users {
  unknown_block {
    something
  }
}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::UnknownSubElement { name, typ_name, .. }
      if name == "unknown_block" && typ_name == "Table"
  )));
}
