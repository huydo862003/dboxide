use salsa::{Database, tracked};

use super::resolve_element_typ::resolve_element_typ;
use crate::StaticTyp;
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{ElementTyp, Symbol, Typ};

/// Can this element type's namespace transitively reach `target` via `.` traversal?
/// Salsa cycle recovery returns false by default.
#[tracked]
pub fn can_namespace_reach_typ<'db>(
  db: &'db dyn Database,
  typ: ElementTyp<'db>,
  target: ElementTyp<'db>,
) -> bool {
  if typ == target {
    return true;
  }
  for (_, lazy_typ) in typ.get_namespace_members(db) {
    let Some(resolved) = lazy_typ.resolve(db) else {
      continue;
    };
    let elem_typ = as_element_typ(db, resolved);
    let Some(elem_typ) = elem_typ else { continue };
    if elem_typ == target || *can_namespace_reach_typ(db, elem_typ, target) {
      return true;
    }
  }
  false
}

pub fn symbol_namespace_reaches<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
  target: ElementTyp<'db>,
) -> bool {
  match symbol.kind(db) {
    SymbolKind::UserElement(file, node) => {
      if let Some(elem_typ) = resolve_element_typ(db, *file, node) {
        *can_namespace_reach_typ(db, elem_typ, target)
      } else {
        false
      }
    }
    SymbolKind::ElementAlias(primary) => symbol_namespace_reaches(db, *primary, target),
    SymbolKind::UserSchema(..) => true,
    _ => false,
  }
}

/// Unwrap `Typ` to `ElementTyp`, also unwrapping `List[T]` -> `T`
fn as_element_typ<'db>(db: &'db dyn Database, typ: Typ<'db>) -> Option<ElementTyp<'db>> {
  match typ {
    Typ::Element(e) => Some(e),
    Typ::List(list_typ) => {
      let elem = (*list_typ.elem(db))?;
      let resolved = elem.resolve(db)?;
      match resolved {
        Typ::Element(e) => Some(e),
        _ => None,
      }
    }
    _ => None,
  }
}
