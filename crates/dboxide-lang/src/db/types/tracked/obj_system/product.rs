use std::collections::BTreeMap;

use salsa::{Database, tracked};

use crate::db::types::interned::lazy_typ::LazyTyp;

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// Anonymous product type: `{ name: string, age: integer }`
#[tracked]
pub struct ProductTyp<'db> {
  pub name: Option<String>,
  pub fields: BTreeMap<String, LazyTyp<'db>>,
}

impl<'db> StaticTyp<'db> for ProductTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "product".to_string()
  }

  fn get_fields(&self, db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    self.fields(db).clone()
  }
}

impl<'db> RuntimeObj<'db> for ProductTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Product(*self)
  }
}

#[tracked]
pub struct ProductObj<'db> {
  #[returns(copy)]
  pub product_typ: Typ<'db>,
  #[returns(ref)]
  pub fields: BTreeMap<String, super::Obj<'db>>,
}

impl<'db> RuntimeObj<'db> for ProductObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    self.product_typ(db)
  }

  fn get_field(&self, db: &'db dyn Database, key: &str) -> Option<Typ<'db>> {
    self.fields(db).get(key).map(|obj| obj.get_typ(db))
  }

  fn get_owned_field(&self, db: &'db dyn Database, key: &str) -> Option<super::Obj<'db>> {
    self.fields(db).get(key).copied()
  }
}
