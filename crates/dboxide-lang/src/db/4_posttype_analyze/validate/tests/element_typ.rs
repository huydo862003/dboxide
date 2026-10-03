use super::*;

#[test]
fn element_typ_unresolved_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

Nonexistent foo {}
"#,
    metatype(),
  );

  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::UnresolvedElementTyp { name, .. } if name == "Nonexistent"
  )));
}
