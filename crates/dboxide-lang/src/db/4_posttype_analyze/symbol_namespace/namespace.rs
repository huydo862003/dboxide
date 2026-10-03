use salsa::Database;

use crate::Typ;
use crate::ast::{AstNode, BlockElementDeclaration, RedNode};
use crate::db::posttype_analyze::{declare_element_symbols, declare_field_line_symbols};
use crate::db::pretype_analyze::{get_declaration_typ, pretype_get_symbol_namespace};
use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind};
use crate::db::types::{File, Symbol};

/// Returns the namespace members for a symbol in posttype context (extends pretype with element children)
pub fn posttype_get_symbol_namespace<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> StaticScopeMembers<'db> {
  match symbol.kind(db) {
    SymbolKind::UserElement(file, node, _) => collect_element_namespace(db, *file, node),
    SymbolKind::ElementAlias(primary, _) => posttype_get_symbol_namespace(db, *primary),
    _ => pretype_get_symbol_namespace(db, symbol).clone(),
  }
}

/// Collects namespace members for an element instance: declared child elements and labeled field line symbols
fn collect_element_namespace<'db>(
  db: &'db dyn Database,
  file: File,
  element_node: &RedNode,
) -> StaticScopeMembers<'db> {
  let mut members = StaticScopeMembers::default();

  let Some(Typ::Element(element_typ)) = *get_declaration_typ(db, file, element_node.clone()) else {
    return members;
  };

  let Some(block) = BlockElementDeclaration::cast(element_node.clone()) else {
    return members;
  };
  let Some(body) = block.body() else {
    return members;
  };

  for nested in body.element_declarations() {
    let declared = declare_element_symbols(db, file, nested.syntax());
    for symbol in declared {
      members.insert(db, symbol);
    }
  }

  let field_nested_typs = &element_typ.typ_nested_typs(db).field_nested_typs;
  if !field_nested_typs.is_empty() {
    for field in body.fields() {
      for symbol in declare_field_line_symbols(db, file, &field, field_nested_typs) {
        members.insert(db, symbol);
      }
    }
  }

  members
}
