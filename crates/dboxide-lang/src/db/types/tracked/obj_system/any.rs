use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

#[tracked]
pub struct AnyTyp<'db> {}

impl<'db> StaticTyp<'db> for AnyTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "any".to_string()
  }

  fn parent_typ(&self, _db: &'db dyn Database) -> Option<Typ<'db>> {
    None
  }
}

impl<'db> RuntimeObj<'db> for AnyTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Any(*self)
  }
}
