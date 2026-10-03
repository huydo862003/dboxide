mod actual_typ;
mod expected_typ;
#[cfg(test)]
mod tests;

pub use expected_typ::get_node_expected_typ;

use salsa::Database;

use crate::ast::RedNode;
use crate::db::posttype_analyze::posttype_get_scope_members;
use crate::db::pretype_analyze::get_declaration_typ;
use crate::db::pretype_evaluate_typ::pretype_evaluate_typ_symbol;
use crate::db::types::interned::symbol::{SymbolCategoryKind, SymbolKind};
use crate::db::types::{File, FuncTyp, Obj, Symbol, Typ};
use crate::get_node_static_scope;

/// Resolve the type a symbol represents
pub(super) fn resolve_symbol_typ<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> Option<Typ<'db>> {
  match symbol.kind(db) {
    SymbolKind::UserElement(element_file, element_node, _) => {
      *get_declaration_typ(db, *element_file, element_node.clone())
    }
    SymbolKind::ElementAlias(primary, _) => resolve_symbol_typ(db, *primary),
    _ => match *pretype_evaluate_typ_symbol(db, symbol) {
      Some(Obj::Typ(typ)) => Some(typ),
      Some(Obj::Func(func_obj)) => Some(Typ::Func(func_obj.func_typ(db))),
      _ => None,
    },
  }
}

/// Extract FuncTyp from a symbol that represents a function
pub(super) fn extract_func_typ<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> Option<FuncTyp<'db>> {
  match *pretype_evaluate_typ_symbol(db, symbol) {
    Some(Obj::Func(func_obj)) => Some(func_obj.func_typ(db)),
    Some(Obj::Typ(Typ::Func(func_typ))) => Some(func_typ),
    _ => None,
  }
}

/// Collect FuncTyp from all operator overloads visible from the given node
pub(super) fn collect_operator_overloads<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
  op: &str,
) -> Vec<FuncTyp<'db>> {
  let mut result = Vec::new();
  let mut current_scope = Some(*get_node_static_scope(db, file, node.clone()));
  while let Some(scope) = current_scope {
    let members = posttype_get_scope_members(db, scope);
    for symbol in members.lookup_symbols(db, op, SymbolCategoryKind::Operator) {
      if let Some(func_typ) = extract_func_typ(db, symbol)
        && !result.contains(&func_typ)
      {
        result.push(func_typ);
      }
    }
    current_scope = scope.get_parent(db);
  }
  result
}
