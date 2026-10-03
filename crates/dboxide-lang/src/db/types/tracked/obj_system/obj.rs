use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

#[tracked]
pub struct ObjTyp<'db> {}

impl<'db> StaticTyp<'db> for ObjTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "obj".to_string()
  }

  fn parent_typ(&self, _db: &'db dyn Database) -> Option<Typ<'db>> {
    None
  }
}

impl<'db> RuntimeObj<'db> for ObjTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Obj(*self)
  }
}
