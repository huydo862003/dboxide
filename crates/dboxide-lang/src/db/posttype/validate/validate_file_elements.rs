use salsa::{Database, tracked};

use crate::ast::{AstNode, SourceFile};
use crate::db::prepost::get_file_symbols::get_file_symbols_pretype;
use crate::db::pretype::evaluate_typ::evaluate_typ_symbol;
use crate::db::types::interned::symbol::{Symbol, SymbolKind};
use crate::db::types::{Diagnostics, File, Obj, Typ};
use crate::diagnostics::DiagnosticKind;
use crate::parse_file::parse_file;

use super::validate_element_fields::validate_element_fields;
use crate::db::prepost::get_file_symbols::get_file_symbols_posttype;

// Validate all element instances in a file
#[tracked]
pub fn validate_file_elements(db: &dyn Database, file: File) {
  // Trigger file-level diagnostics (duplicate labels, etc)
  get_file_symbols_posttype(db, file, false);

  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return;
  };

  // Validate type definitions
  let pretype_scope = get_file_symbols_pretype(db, file, false);
  for (_, symbol) in pretype_scope.iter() {
    if matches!(symbol.kind(db), SymbolKind::UserTyp(..)) {
      validate_typ_definition(db, *symbol);
    }
  }

  for element_declaration in source_file.element_declarations() {
    let Some(typ) = element_declaration.typ() else {
      continue;
    };
    // Skip type definitions
    if typ.text().trim().eq_ignore_ascii_case("type") {
      continue;
    }
    validate_element_fields(db, file, element_declaration.syntax());
  }
}

fn validate_typ_definition(db: &dyn Database, symbol: Symbol<'_>) {
  let Some(Obj::Typ(Typ::Element(element_typ))) = *evaluate_typ_symbol(db, symbol) else {
    return;
  };
  let attrs = element_typ.typ_element_attributes(db);
  let Some(rest) = attrs.rest else { return };
  let resolved = rest.resolve(db);
  if !matches!(resolved, Some(Typ::Map(_))) {
    let SymbolKind::UserTyp(_, node) = symbol.kind(db) else {
      return;
    };
    let start = node.offset();
    salsa::Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::RestAttributeMustBeMap.into_error(start, start + node.text_len()),
      ),
      db,
    );
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use itertools::Either;

  use crate::db::DboxideDatabase;
  use crate::db::posttype::validate::validate_file_elements::validate_file_elements;
  use crate::db::test_utils::*;
  use crate::db::types::Diagnostics;
  use crate::db::types::interned::symbol::VirtualModuleKind;
  use crate::diagnostics::DiagnosticKind;

  fn metatype() -> HashMap<String, Either<VirtualModuleKind, crate::db::types::File>> {
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )])
  }

  #[test]
  fn valid_element_body_no_diagnostics() {
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
  fn duplicate_unique_label_emits_diagnostic() {
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
}
