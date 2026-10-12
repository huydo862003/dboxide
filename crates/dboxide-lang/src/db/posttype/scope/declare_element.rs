use salsa::Database;

use crate::ast::{AstNode, ElementDeclaration, RedNode};
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{File, Symbol};

use crate::db::posttype::resolve::resolve_element_typ::resolve_element_typ;

// Given an element instance node, extract declared symbols from label fields
pub fn declare_element_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  element_node: &RedNode,
) -> Vec<Symbol<'db>> {
  let mut symbols = Vec::new();

  let Some(element) = ElementDeclaration::cast(element_node.clone()) else {
    return symbols;
  };

  let Some(element_typ) = resolve_element_typ(db, file, element_node) else {
    return symbols;
  };

  let file_path = file.filepath(db).display().to_string();
  let labels = element_typ.label_definitions(db);

  let has_primary_label = labels.iter().any(|label| !label.is_alias);
  if has_primary_label
    && let Some(name) = element.declaration_name()
    && !name.is_empty()
  {
    let symbol = Symbol::new(
      db,
      SymbolKind::UserElement(file, element_node.clone()),
      name.clone(),
      format!("{file_path}::{name}"),
    );
    symbols.push(symbol);
  }

  for _ in labels.iter().filter(|label| label.is_alias) {
    let Some(alias_node) = element.alias() else {
      continue;
    };
    let Some(alias_name_node) = alias_node.name() else {
      continue;
    };
    let alias_name = alias_name_node.text().trim().to_string();
    if alias_name.is_empty() {
      continue;
    }
    let Some(&primary_symbol) = symbols.first() else {
      continue;
    };
    let symbol = Symbol::new(
      db,
      SymbolKind::ElementAlias(primary_symbol),
      alias_name.clone(),
      format!("{file_path}::@alias::{alias_name}"),
    );
    symbols.push(symbol);
  }

  symbols
}
