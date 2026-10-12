use salsa::{Database, tracked};

use crate::ast::{AstNode, SourceFile};
use crate::db::types::File;
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
