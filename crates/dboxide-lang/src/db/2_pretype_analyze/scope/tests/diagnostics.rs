use std::collections::HashMap;

use itertools::Either;

use super::*;
use crate::ast::{AstNode, BlockElementDeclaration, SourceFile};
use crate::db::pretype_analyze::pretype_get_file_symbols;
use crate::db::types::{Diagnostics, StaticScope, StaticScopeKind};
use crate::diagnostics::DiagnosticKind;
use crate::parse_file::parse_file;

fn get_file_diagnostics(db: &DboxideDatabase, file: File) -> Vec<&Diagnostics> {
  pretype_get_file_symbols::accumulated::<Diagnostics>(db, file, false)
}

fn get_element_body_scope_diagnostics(db: &DboxideDatabase, file: File) -> Vec<&Diagnostics> {
  // Get the body node of the last element declaration in the file (skip type declarations)
  let root = parse_file(db, file).ast(db).value(db).node.clone();
  let source = SourceFile::cast(root).unwrap();
  let element = source
    .element_declarations()
    .find(|element| !element.is_typ_declaration())
    .unwrap();
  let body = BlockElementDeclaration::cast(element.syntax().clone())
    .and_then(|body| body.body())
    .unwrap();
  let scope = StaticScope::new(
    db,
    StaticScopeKind::ElementNamespace(file, body.syntax().clone()),
  );
  pretype_get_scope_members::accumulated::<Diagnostics>(db, scope)
}

fn get_type_body_scope_diagnostics(db: &DboxideDatabase, file: File) -> Vec<&Diagnostics> {
  // Get the body node of the first type declaration in the file
  let root = parse_file(db, file).ast(db).value(db).node.clone();
  let source = SourceFile::cast(root).unwrap();
  let element = source.element_declarations().next().unwrap();
  let body = BlockElementDeclaration::cast(element.syntax().clone())
    .and_then(|body| body.body())
    .unwrap();
  let scope = StaticScope::new(
    db,
    StaticScopeKind::TypNamespace(file, body.syntax().clone()),
  );
  pretype_get_scope_members::accumulated::<Diagnostics>(db, scope)
}

mod declaration {
  use super::*;

  #[test]
  fn declaration_duplicate_type_in_file_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
type Foo {
  field string
}
type Foo {
  other integer
}
"#,
      HashMap::new(),
    );
    let diagnostics = get_file_diagnostics(&db, file);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateTypeDeclaration { name, .. } if name == "Foo"
      )),
      "expected DuplicateTypeDeclaration for 'Foo'"
    );
  }

  #[test]
  fn declaration_duplicate_equality_in_file_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
type Alias = string
type Alias = integer
"#,
      HashMap::new(),
    );
    let diagnostics = get_file_diagnostics(&db, file);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateTypeDeclaration { name, .. } if name == "Alias"
      )),
      "expected DuplicateTypeDeclaration for 'Alias'"
    );
  }

  #[test]
  fn declaration_unique_types_in_file_no_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
type Foo { field string }
type Bar { field integer }
type Alias = string
"#,
      HashMap::new(),
    );
    let diagnostics = get_file_diagnostics(&db, file);
    assert!(
      !diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateTypeDeclaration { .. }
      )),
      "unexpected DuplicateTypeDeclaration diagnostic"
    );
  }

  #[test]
  fn declaration_duplicate_type_in_element_body_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
type Table {
  field string
}
Table users {
  type Inner { field string }
  type Inner { other integer }
}
"#,
      HashMap::new(),
    );
    let diagnostics = get_element_body_scope_diagnostics(&db, file);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateTypeDeclaration { name, .. } if name == "Inner"
      )),
      "expected DuplicateTypeDeclaration for 'Inner' in element body"
    );
  }

  #[test]
  fn declaration_duplicate_type_in_type_body_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
type Table {
  type Inner { field string }
  type Inner { other integer }
}
"#,
      HashMap::new(),
    );
    let diagnostics = get_type_body_scope_diagnostics(&db, file);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateTypeDeclaration { name, .. } if name == "Inner"
      )),
      "expected DuplicateTypeDeclaration for 'Inner' in type body"
    );
  }

  #[test]
  fn declaration_first_occurrence_recorded_correctly() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"type First { a string }
type First { b integer }
"#,
      HashMap::new(),
    );
    let diagnostics = get_file_diagnostics(&db, file);
    let diag = diagnostics
      .iter()
      .find(|diagnostic| matches!(&diagnostic.0.kind, DiagnosticKind::DuplicateTypeDeclaration { name, .. } if name == "First"))
      .expect("expected DuplicateTypeDeclaration");
    // The diagnostic is on the second declaration; the first_start/first_end point at the first
    if let DiagnosticKind::DuplicateTypeDeclaration {
      first_start,
      first_end,
      ..
    } = &diag.0.kind
    {
      assert!(*first_start < *first_end, "first range should be valid");
      assert!(
        *first_end < diag.0.start_offset,
        "first declaration should precede the duplicate"
      );
    } else {
      panic!("unexpected diagnostic kind");
    }
  }
}

mod import {
  use super::*;

  fn make_dep(db: &DboxideDatabase, src: &str) -> File {
    create_file(db, "dep.dbml", src, HashMap::new())
  }

  fn make_main(db: &DboxideDatabase, dep: File, src: &str) -> File {
    create_file(
      db,
      "main.dbml",
      src,
      HashMap::from([("./dep".to_string(), Either::Right(dep))]),
    )
  }

  #[test]
  fn import_conflicts_with_local_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let dep = make_dep(&db, "type Foo { field string }");
    let main = make_main(
      &db,
      dep,
      r#"
type Foo { other integer }
use { type Foo } from './dep'
"#,
    );
    let diagnostics = get_file_diagnostics(&db, main);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::ImportConflictsWithLocal { name } if name == "Foo"
      )),
      "expected ImportConflictsWithLocal for 'Foo'"
    );
  }

  #[test]
  fn import_same_symbol_twice_no_diagnostic() {
    let db = DboxideDatabase::default();
    let dep = make_dep(&db, "type Foo { field string }");
    let dep2 = create_file(
      &db,
      "dep2.dbml",
      "use { type Foo } from './dep'",
      HashMap::from([("./dep".to_string(), Either::Right(dep))]),
    );
    let main = create_file(
      &db,
      "main.dbml",
      r#"
use { type Foo } from './dep'
use { type Foo } from './dep2'
"#,
      HashMap::from([
        ("./dep".to_string(), Either::Right(dep)),
        ("./dep2".to_string(), Either::Right(dep2)),
      ]),
    );
    let diagnostics = get_file_diagnostics(&db, main);
    assert!(
      !diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::ConflictingImport { .. } | DiagnosticKind::ImportConflictsWithLocal { .. }
      )),
      "unexpected import conflict diagnostic for same underlying symbol"
    );
  }

  #[test]
  fn import_different_symbols_same_name_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let dep1 = create_file(&db, "dep1.dbml", "type Foo { a string }", HashMap::new());
    let dep2 = create_file(&db, "dep2.dbml", "type Foo { b integer }", HashMap::new());
    let main = create_file(
      &db,
      "main.dbml",
      r#"
use { type Foo } from './dep1'
use { type Foo } from './dep2'
"#,
      HashMap::from([
        ("./dep1".to_string(), Either::Right(dep1)),
        ("./dep2".to_string(), Either::Right(dep2)),
      ]),
    );
    let diagnostics = get_file_diagnostics(&db, main);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::ConflictingImport { name } if name == "Foo"
      )),
      "expected ConflictingImport for 'Foo'"
    );
  }
}
