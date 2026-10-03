use salsa::{Database, tracked};

use crate::ast::AstNode;
use crate::ast::SourceFile;

use crate::db::types::File;
use crate::parse_file::parse_file;

/// Extract all import specifiers from a file's `use`/`reuse` declarations
#[tracked]
pub fn get_use_specifiers(db: &dyn Database, file: File) -> Vec<String> {
  let parse_result = parse_file(db, file);
  let root_node = parse_result.ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return Vec::new();
  };

  source_file
    .use_declarations()
    .filter_map(|use_declaration| use_declaration.module_path())
    .collect()
}
