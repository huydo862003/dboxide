use salsa::{Database, tracked};

use crate::ast::{AstNode, SourceFile};
use crate::db::types::File;
use crate::parse_file::parse_file;

use super::posttype_check_element::check_element;

// type-check all element instances in a file
#[tracked]
pub fn check_file(db: &dyn Database, file: File) {
  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return;
  };

  for element_declaration in source_file.element_declarations() {
    let Some(typ) = element_declaration.typ() else {
      continue;
    };
    // skip type definitions
    if typ.text().trim().eq_ignore_ascii_case("type") {
      continue;
    }
    check_element(db, file, element_declaration.syntax());
  }
}
