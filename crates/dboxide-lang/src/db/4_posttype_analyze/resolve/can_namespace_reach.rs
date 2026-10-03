use salsa::{Database, tracked};

use crate::StaticTyp;
use crate::db::types::ElementTyp;
use crate::db::types::Typ;

/// Returns true if this element type's namespace transitively reaches `target` via `.` traversal
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
    let Some(resolved_typ) = lazy_typ.resolve(db) else {
      continue;
    };
    let Typ::Element(element_typ) = resolved_typ else {
      continue;
    };
    if element_typ == target || *can_namespace_reach_typ(db, element_typ, target) {
      return true;
    }
  }
  // Also check the expanded field item type (`columns [expand]` exposes Column items by name)
  if let Some(item_lazy) = typ.typ_namespace(db).expanded_item_typ
    && let Some(Typ::Element(item_element_typ)) = item_lazy.resolve(db)
    && (item_element_typ == target || *can_namespace_reach_typ(db, item_element_typ, target))
  {
    return true;
  }
  false
}
