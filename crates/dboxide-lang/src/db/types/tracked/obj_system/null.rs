use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

#[tracked]
pub struct NullTyp<'db> {}

impl<'db> StaticTyp<'db> for NullTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "null".to_string()
  }
}

impl<'db> RuntimeObj<'db> for NullTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Null(*self)
  }
}

#[tracked]
pub struct NullObj<'db> {}

impl<'db> RuntimeObj<'db> for NullObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    NullTyp::new(db).into()
  }

  fn to_display_string(&self, _db: &'db dyn Database) -> String {
    "null".to_string()
  }
}
