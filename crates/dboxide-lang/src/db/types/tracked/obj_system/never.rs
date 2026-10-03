use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// Bottom type: subtype of everything, nothing is a subtype of it
#[tracked]
pub struct NeverTyp<'db> {}

impl<'db> StaticTyp<'db> for NeverTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "never".to_string()
  }

  fn parent_typ(&self, _db: &'db dyn Database) -> Option<Typ<'db>> {
    None
  }
}

impl<'db> RuntimeObj<'db> for NeverTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Never(*self)
  }
}
