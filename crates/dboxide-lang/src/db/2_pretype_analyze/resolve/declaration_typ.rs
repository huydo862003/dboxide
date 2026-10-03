use salsa::{Database, tracked};

use crate::ast::{AstNode, ElementDeclaration, RedNode};
use crate::db::pretype_evaluate_typ::pretype_evaluate_typ_symbol;
use crate::db::types::File;
use crate::{Obj, Typ, TypTyp};

use super::referee::pretype_resolve_referee;

/// Returns the declared `Typ` for an element declaration node
#[tracked]
pub fn get_declaration_typ<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
) -> Option<Typ<'db>> {
  let element = ElementDeclaration::cast(node)?;
  let fragment = element.typ()?.name()?;
  let typ_node = fragment.node();

  if element.is_typ_declaration() {
    return Some(Typ::TypTyp(TypTyp::new(db)));
  }

  let symbol = (*pretype_resolve_referee(db, file, typ_node.clone()))?;
  match *pretype_evaluate_typ_symbol(db, symbol) {
    Some(Obj::Typ(Typ::Element(element_typ))) => Some(element_typ.into()),
    _ => None,
  }
}
