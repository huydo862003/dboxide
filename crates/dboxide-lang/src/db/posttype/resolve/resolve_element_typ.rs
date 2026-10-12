use salsa::Database;

use crate::ast::{AstNode, ElementDeclaration, RedNode};
use crate::db::prepost::resolve_name_pretype as resolve_name;
use crate::db::pretype::evaluate_typ::evaluate_typ_symbol;
use crate::db::types::{ElementTyp, File, Obj, Typ};

// Resolve an element node's type name to its ElementTyp definition
// Returns None if the node is a `type` declaration or the type cannot be resolved
pub fn resolve_element_typ<'db>(
  db: &'db dyn Database,
  file: File,
  element_node: &RedNode,
) -> Option<ElementTyp<'db>> {
  let element = ElementDeclaration::cast(element_node.clone())?;
  let typ_node = element.typ()?;
  let fragments = typ_node.collect_path_fragments();
  let last = fragments.last()?;
  if last.eq_ignore_ascii_case("type") {
    return None;
  }
  let typ_symbol = (*resolve_name(db, file, element_node.clone(), last.clone()))?;
  let Some(Obj::Typ(Typ::Element(element_typ))) = *evaluate_typ_symbol(db, typ_symbol) else {
    return None;
  };
  Some(element_typ)
}
