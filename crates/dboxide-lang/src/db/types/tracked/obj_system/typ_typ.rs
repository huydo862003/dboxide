use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// The metatype: the type of all types. Instance of itself.
#[tracked]
pub struct TypTyp<'db> {}

impl<'db> StaticTyp<'db> for TypTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "type".to_string()
  }

  fn is_typ(&self, _db: &'db dyn Database) -> bool {
    true
  }

  fn runtime_typ(&self, _db: &'db dyn Database) -> Option<Typ<'db>> {
    Some((*self).into())
  }

  fn parent_typ(&self, _db: &'db dyn Database) -> Option<Typ<'db>> {
    None
  }
}

impl<'db> RuntimeObj<'db> for TypTyp<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    Typ::TypTyp(TypTyp::new(db))
  }
}
