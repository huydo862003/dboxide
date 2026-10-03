use super::*;
use crate::db::posttype_analyze::posttype_get_file_symbols;
use crate::db::types::Diagnostics;
use crate::diagnostics::DiagnosticKind;
use crate::validate_file_elements;

fn get_file_diagnostics(db: &DboxideDatabase, file: File) -> Vec<&Diagnostics> {
  posttype_get_file_symbols::accumulated::<Diagnostics>(db, file, false)
}

fn get_validate_diagnostics(db: &DboxideDatabase, file: File) -> Vec<&Diagnostics> {
  validate_file_elements::accumulated::<Diagnostics>(db, file)
}

mod field_declaration {
  use super::*;

  fn make_table_file(db: &DboxideDatabase, src: &str) -> File {
    create_file(
      db,
      "test.dbml",
      &format!(
        r#"
use * from 'metatype'

type Table [element] {{
  type Column [field] {{
    col_name string [arg: 0, label, unique]
  }}
}}

{src}
"#
      ),
      metatype(),
    )
  }

  #[test]
  fn field_declaration_unique_names_no_diagnostic() {
    let db = DboxideDatabase::default();
    let file = make_table_file(
      &db,
      r#"
Table users {
  id
  email
}
"#,
    );
    let diagnostics = get_validate_diagnostics(&db, file);
    assert!(
      !diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateFieldDeclaration { .. }
      )),
      "unexpected DuplicateFieldDeclaration diagnostic"
    );
  }

  #[test]
  fn field_declaration_duplicate_name_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = make_table_file(
      &db,
      r#"
Table users {
  id
  email
  id
}
"#,
    );
    let diagnostics = get_validate_diagnostics(&db, file);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateFieldDeclaration { name, .. } if name == "id"
      )),
      "expected DuplicateFieldDeclaration for 'id'"
    );
  }
}

mod import_kind {
  use super::*;

  fn make_dep_without_unique_label(db: &DboxideDatabase) -> File {
    create_file(
      db,
      "dep.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name string
}

Table users {}
"#,
      metatype(),
    )
  }

  fn make_dep_with_unique_label(db: &DboxideDatabase) -> File {
    create_file(
      db,
      "dep.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
"#,
      metatype(),
    )
  }

  fn make_main(db: &DboxideDatabase, dep: File, src: &str) -> File {
    create_file(
      db,
      "main.dbml",
      src,
      HashMap::from([
        ("./dep".to_string(), Either::Right(dep)),
        (
          "metatype".to_string(),
          Either::Left(VirtualModuleKind::Metatype),
        ),
      ]),
    )
  }

  #[test]
  fn import_kind_without_unique_label_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let dep = make_dep_without_unique_label(&db);
    let main = make_main(
      &db,
      dep,
      r#"
use { type Table } from './dep'
use { Table users } from './dep'
"#,
    );
    let diagnostics = get_file_diagnostics(&db, main);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::ImportKindMustBeUnique { kind_name } if kind_name == "Table"
      )),
      "expected ImportKindMustBeUnique for 'Table'"
    );
  }

  #[test]
  fn import_kind_with_unique_label_no_diagnostic() {
    let db = DboxideDatabase::default();
    let dep = make_dep_with_unique_label(&db);
    let main = make_main(
      &db,
      dep,
      r#"
use { type Table } from './dep'
use { Table users } from './dep'
"#,
    );
    let diagnostics = get_file_diagnostics(&db, main);
    assert!(
      !diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::ImportKindMustBeUnique { .. }
      )),
      "unexpected ImportKindMustBeUnique diagnostic"
    );
  }
}
