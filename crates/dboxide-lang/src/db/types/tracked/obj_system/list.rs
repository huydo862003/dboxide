use salsa::{Database, tracked};

use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::typ_param::{TypParams, TypVariable, Variance};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// `List[V]`
#[tracked]
pub struct ListTyp<'db> {
  pub elem: Option<LazyTyp<'db>>,
}

impl<'db> StaticTyp<'db> for ListTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "List".to_string()
  }

  fn arity(&self, db: &'db dyn Database) -> usize {
    if self.elem(db).is_some() { 0 } else { 1 }
  }

  fn typ_params(&self, db: &'db dyn Database) -> Option<TypParams<'db>> {
    if self.elem(db).is_some() {
      return None;
    }
    Some(TypParams::new(
      db,
      vec![TypVariable::new(db, None, Variance::Covariant)],
      vec![],
    ))
  }

  fn instantiate(&self, db: &'db dyn Database, args: Vec<LazyTyp<'db>>) -> Option<Typ<'db>> {
    if args.len() != 1 {
      return None;
    }
    Some(ListTyp::new(db, Some(args[0])).into())
  }
}

impl<'db> RuntimeObj<'db> for ListTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::List(*self)
  }

  fn index(&self, db: &'db dyn Database, key: &super::Obj<'db>) -> Option<super::Obj<'db>> {
    let arg_typ = key.as_typ()?;
    self
      .instantiate(db, vec![LazyTyp::eager(arg_typ)])
      .map(super::Obj::Typ)
  }
}

#[tracked]
pub struct ListObj<'db> {
  #[returns(ref)]
  pub items: Vec<super::Obj<'db>>,
}

impl<'db> RuntimeObj<'db> for ListObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    ListTyp::new(db, None).into()
  }
}
