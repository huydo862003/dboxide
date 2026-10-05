use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

#[tracked]
pub struct ColorTyp<'db> {}

impl<'db> StaticTyp<'db> for ColorTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "color".to_string()
  }
}

impl<'db> RuntimeObj<'db> for ColorTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Color(*self)
  }
}
