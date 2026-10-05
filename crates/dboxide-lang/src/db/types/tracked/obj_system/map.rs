use salsa::{Database, tracked};

use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::typ_param::{TypParams, TypVariable, Variance};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// `Map[K, V]`
#[tracked]
pub struct MapTyp<'db> {
  pub key: Option<LazyTyp<'db>>,
  pub value: Option<LazyTyp<'db>>,
}

impl<'db> StaticTyp<'db> for MapTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "Map".to_string()
  }

  fn arity(&self, db: &'db dyn Database) -> usize {
    match (self.key(db).is_some(), self.value(db).is_some()) {
      (true, true) => 0,
      _ => 2,
    }
  }

  fn typ_params(&self, db: &'db dyn Database) -> Option<TypParams<'db>> {
    if self.key(db).is_some() && self.value(db).is_some() {
      return None;
    }
    Some(TypParams::new(
      db,
      vec![
        TypVariable::new(db, None, Variance::Covariant),
        TypVariable::new(db, None, Variance::Covariant),
      ],
      vec![],
    ))
  }

  fn instantiate(&self, db: &'db dyn Database, args: Vec<LazyTyp<'db>>) -> Option<Typ<'db>> {
    if args.len() != 2 {
      return None;
    }
    Some(MapTyp::new(db, Some(args[0]), Some(args[1])).into())
  }
}

impl<'db> RuntimeObj<'db> for MapTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Map(*self)
  }
}

#[tracked]
pub struct MapObj<'db> {
  #[returns(ref)]
  pub entries: Vec<(super::Obj<'db>, super::Obj<'db>)>,
}

impl<'db> RuntimeObj<'db> for MapObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    MapTyp::new(db, None, None).into()
  }
}
