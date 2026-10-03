use std::collections::BTreeMap;

use salsa::{Database, tracked};

use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::typ_param::{TypParams, TypVariable, Variance};
use crate::{IntegerObj, Obj, ObjTyp};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};
use super::func::{FuncObj, FuncTyp};

/// `List[V]`
#[tracked]
pub struct ListTyp<'db> {
  pub element: Option<LazyTyp<'db>>,
}

impl<'db> StaticTyp<'db> for ListTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "List".to_string()
  }

  fn arity(&self, db: &'db dyn Database) -> usize {
    if self.element(db).is_some() { 0 } else { 1 }
  }

  fn typ_params(&self, db: &'db dyn Database) -> Option<TypParams<'db>> {
    if self.element(db).is_some() {
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

  fn runtime_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, FuncObj<'db>> {
    let any = LazyTyp::eager(ObjTyp::new(db).into());
    let signature = FuncSignature::new(db, vec![any], any);
    let func_typ = FuncTyp::new(db, signature);
    let mut vtable = BTreeMap::new();
    for name in [
      "filter", "map", "every", "some", "flatMap", "indices", "concat",
    ] {
      vtable.insert(
        name.to_string(),
        FuncObj::new(db, name.to_string(), func_typ, None, None),
      );
    }
    vtable
  }
}

impl<'db> RuntimeObj<'db> for ListTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::List(*self)
  }

  fn index(&self, db: &'db dyn Database, key: &Obj<'db>) -> Option<Obj<'db>> {
    let arg_typ = key.as_typ()?;
    self
      .instantiate(db, vec![LazyTyp::eager(arg_typ)])
      .map(Obj::Typ)
  }
}

#[tracked]
pub struct ListObj<'db> {
  #[returns(ref)]
  pub items: Vec<Obj<'db>>,
}

impl<'db> RuntimeObj<'db> for ListObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    ListTyp::new(db, None).into()
  }

  fn get_owned_field(&self, db: &'db dyn Database, key: &str) -> Option<Obj<'db>> {
    match key {
      "length" => Some(Obj::Integer(IntegerObj::new(
        db,
        self.items(db).len() as i64,
      ))),
      _ => None,
    }
  }

  fn index(&self, db: &'db dyn Database, key: &Obj<'db>) -> Option<Obj<'db>> {
    if let Obj::Integer(int) = key {
      let index = int.value(db);
      if index >= 0 {
        return self.items(db).get(index as usize).copied();
      }
    }
    None
  }
}
