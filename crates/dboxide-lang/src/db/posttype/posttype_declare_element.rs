use salsa::Database;

use crate::ast::{AstNode, ElementDeclaration, RedNode};
use crate::db::pretype::pretype_evaluate::pretype_evaluate_typ_symbol::evaluate_typ_symbol;
use crate::db::pretype::pretype_resolve_name::resolve_name;
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{File, Obj, Symbol, Typ};

// given an element instance node (e.g. `Table users { }`), extract declared symbols
// from label fields on the resolved ElementTyp
pub fn declare_element_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  element_node: &RedNode,
) -> Vec<Symbol<'db>> {
  let mut symbols = Vec::new();

  let Some(element) = ElementDeclaration::cast(element_node.clone()) else {
    return symbols;
  };

  let Some(typ_node) = element.typ() else {
    return symbols;
  };

  // skip `type` declarations - those are handled by pretype
  let fragments = typ_node.collect_path_fragments();
  let Some(last_fragment) = fragments.last() else {
    return symbols;
  };
  if last_fragment.eq_ignore_ascii_case("type") {
    return symbols;
  }

  // resolve the element type name to a Symbol, then evaluate to ElementTyp
  let Some(typ_symbol) = *resolve_name(db, file, element_node.clone(), last_fragment.clone())
  else {
    return symbols;
  };
  let Some(Obj::Typ(Typ::Element(element_typ))) = *evaluate_typ_symbol(db, typ_symbol) else {
    return symbols;
  };

  let file_path = file.filepath(db).display().to_string();
  let labels = element_typ.label_definitions(db);

  // process primary labels first, then aliases (aliases reference the primary symbol)
  let has_primary_label = labels.iter().any(|l| !l.is_alias);
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

  for _ in labels.iter().filter(|l| l.is_alias) {
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
