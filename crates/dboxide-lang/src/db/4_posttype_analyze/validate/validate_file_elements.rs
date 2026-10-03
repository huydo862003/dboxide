use salsa::{Accumulator, Database, tracked};

use crate::ast::{AstNode, SourceFile};
use crate::db::posttype_analyze::posttype_get_file_symbols;
use crate::db::posttype_evaluate_element::evaluate_file;
use crate::db::pretype_analyze::pretype_get_file_symbols;
use crate::db::pretype_evaluate_typ::pretype_evaluate_typ_symbol;
use crate::db::types::interned::symbol::{Symbol, SymbolKind};
use crate::db::types::{Diagnostics, File, Obj, Typ};
use crate::diagnostics::DiagnosticKind;
use crate::parse_file::parse_file;

use super::validate_constraints::validate_element_constraints;
use super::validate_element_fields::validate_element_fields;

/// Validates all element instances in a file
#[tracked]
pub fn validate_file_elements(db: &dyn Database, file: File) {
  // Trigger file-level diagnostics (duplicate labels, etc)
  posttype_get_file_symbols(db, file, false);

  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return;
  };

  // Validate type definitions
  let pretyp_scope = pretype_get_file_symbols(db, file, false);
  for (_, symbol) in pretyp_scope.iter() {
    if matches!(symbol.kind(db), SymbolKind::UserTyp(..)) {
      validate_typ_definition(db, *symbol);
    }
  }

  // Validate element structure (field counts, unknown settings/attributes/sub-elements, CSV validation)
  for element_declaration in source_file.element_declarations() {
    if element_declaration.is_typ_declaration() {
      continue;
    }
    validate_element_fields(db, file, element_declaration.syntax());
  }

  // Validate constraints on interpreted elements (single source of truth)
  for element_obj in evaluate_file(db, file).iter() {
    validate_element_constraints(db, file, *element_obj, None);
  }
}

/// Validates invariants on a type definition symbol (e.g. rest attribute must be Map)
fn validate_typ_definition(db: &dyn Database, symbol: Symbol<'_>) {
  let Some(Obj::Typ(Typ::Element(element_typ))) = *pretype_evaluate_typ_symbol(db, symbol) else {
    return;
  };
  let attribues = element_typ.typ_element_attributes(db);
  let Some(rest) = attribues.rest else { return };
  let resolved = rest.resolve(db);
  if !matches!(resolved, Some(Typ::Map(_))) {
    let SymbolKind::UserTyp(_, node, _) = symbol.kind(db) else {
      return;
    };

    Diagnostics::accumulate(
      Diagnostics(
        DiagnosticKind::RestAttributeMustBeMap.into_error(node.start_offset(), node.end_offset()),
      ),
      db,
    );
  }
}
