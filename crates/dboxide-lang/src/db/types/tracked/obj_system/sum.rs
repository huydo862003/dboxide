use salsa::{Database, tracked};

use crate::db::types::interned::lazy_typ::LazyTyp;

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// Union type: `A | B`, `T?` (= `T | null`)
/// Members should be deduplicated and flattened at construction
#[tracked]
pub struct SumTyp<'db> {
  pub members: Vec<Typ<'db>>,
}

impl<'db> SumTyp<'db> {
  /// Create a SumTyp with flattened and deduplicated members
  pub fn create(db: &'db dyn Database, members: Vec<Typ<'db>>) -> Typ<'db> {
    let mut flat = Vec::new();
    for m in members {
      if let Typ::Sum(inner) = m {
        for inner_m in inner.members(db).iter() {
          if !flat.contains(inner_m) {
            flat.push(*inner_m);
          }
        }
      } else if !flat.contains(&m) {
        flat.push(m);
      }
    }
    if flat.len() == 1 {
      return flat.into_iter().next().unwrap();
    }
    SumTyp::new(db, flat).into()
  }
}

impl<'db> StaticTyp<'db> for SumTyp<'db> {
  fn display_name(&self, db: &'db dyn Database) -> String {
    self
      .members(db)
      .iter()
      .map(|member| member.display_name(db))
      .collect::<Vec<_>>()
      .join(" | ")
  }

  /// Distribute field access: (A | B).field = A.field | B.field
  fn lookup_field_typ(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    let mut field_typs = Vec::new();
    for member in self.members(db).iter() {
      let field = member.lookup_field_typ(db, name)?;
      if let Some(typ) = field.as_eager()
        && !field_typs.contains(&typ)
      {
        field_typs.push(typ);
      }
    }
    if field_typs.is_empty() {
      return None;
    }
    if field_typs.len() == 1 {
      return Some(LazyTyp::eager(field_typs.into_iter().next().unwrap()));
    }
    Some(LazyTyp::eager(SumTyp::create(db, field_typs)))
  }
}

impl<'db> RuntimeObj<'db> for SumTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Sum(*self)
  }
}
