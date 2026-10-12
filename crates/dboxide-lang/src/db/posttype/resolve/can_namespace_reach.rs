use salsa::{Database, tracked};

use super::resolve_element_typ::resolve_element_typ;
use crate::StaticTyp;
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{ElementTyp, Symbol, Typ};

/// Can this element type's namespace transitively reach `target` via `.` traversal?
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
    let element_typ = as_element_typ(db, resolved);
    let Some(element_typ) = element_typ else {
      continue;
    };
    if element_typ == target || *can_namespace_reach_typ(db, element_typ, target) {
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
      if let Some(element_typ) = resolve_element_typ(db, *file, node) {
        *can_namespace_reach_typ(db, element_typ, target)
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
      let resolved = (*list_typ.element(db))?.resolve(db)?;
      match resolved {
        Typ::Element(e) => Some(e),
        _ => None,
      }
    }
    _ => None,
  }
}
