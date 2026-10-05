pub mod alias;
pub mod constraints;
pub mod nested_typ_element;

use salsa::{Database, tracked};

use crate::ast::{AstNode, BlockElementDeclaration, EqualityDeclaration, RedNode};
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{File, Obj, Symbol, Typ};

pub use alias::{evaluate_typ_alias_node, evaluate_virtual_typ_kind};
pub use constraints::{
  is_declaration_typ, validate_contiguous_arg_index, validate_declaration_field_labels,
};
pub use nested_typ_element::evaluate_typ_element_node;

#[tracked]
pub fn comp_evaluate_typ_symbol<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> Option<Obj<'db>> {
  match symbol.kind(db) {
    SymbolKind::VirtualTyp(_, kind) => Some(Obj::Typ(evaluate_virtual_typ_kind(db, kind))),
    SymbolKind::UserTyp(file, node) => {
      evaluate_user_typ_kind(db, *file, symbol.name(db), node).map(Obj::Typ)
    }
    SymbolKind::Use(target, _, _) => *comp_evaluate_typ_symbol(db, *target),
    _ => None,
  }
}

fn evaluate_user_typ_kind<'db>(
  db: &'db dyn Database,
  file: File,
  name: &str,
  node: &RedNode,
) -> Option<Typ<'db>> {
  if let Some(equality_declaration) = EqualityDeclaration::cast(node.clone()) {
    return evaluate_typ_alias_node(db, file, &equality_declaration);
  }

  if let Some(block_declaration) = BlockElementDeclaration::cast(node.clone()) {
    return Some(evaluate_typ_element_node(
      db,
      file,
      name,
      &block_declaration,
    ));
  }

  None
}
