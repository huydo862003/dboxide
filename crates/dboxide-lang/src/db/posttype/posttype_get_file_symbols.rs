use salsa::{Database, tracked};

use crate::ast::{AstNode, SourceFile};
use crate::parse_file::parse_file;

use crate::db::pretype::pretype_get_file_usable_symbols::get_file_types_and_schemas;
use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind};
use crate::db::types::{File, Symbol};

use super::posttype_declare_element::declare_element_symbols;

// extends pretype file symbols with functions, operators, and element instance symbols
#[tracked]
pub fn get_file_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  reuses_only: bool,
) -> StaticScopeMembers<'db> {
  let mut members = get_file_types_and_schemas(db, file, reuses_only).clone();

  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return members;
  };

  let file_path = file.filepath(db).display().to_string();

  // collect top-level functions and operators
  for func_declaration in source_file.func_declarations() {
    let Some(name_node) = func_declaration.name() else {
      continue;
    };
    if name_node.is_operator_declaration() {
      let Some(op_symbol) = name_node.operator_symbol() else {
        continue;
      };
      let symbol = Symbol::new(
        db,
        SymbolKind::UserOperator(file, func_declaration.syntax().clone()),
        op_symbol.clone(),
        format!("{file_path}::operator::{op_symbol}"),
      );
      members.insert(db, symbol);
    } else {
      let name = name_node.text().trim().to_string();
      let symbol = Symbol::new(
        db,
        SymbolKind::UserFunction(file, func_declaration.syntax().clone()),
        name.clone(),
        format!("{file_path}::{name}"),
      );
      members.insert(db, symbol);
    }
  }

  // collect element instance symbols from label declarations
  for element_declaration in source_file.element_declarations() {
    let declared = declare_element_symbols(db, file, element_declaration.syntax());
    for symbol in declared {
      members.insert(db, symbol);
    }
  }

  members
}
