use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

#[tracked]
pub struct StringTyp<'db> {}

impl<'db> StaticTyp<'db> for StringTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "string".to_string()
  }
}

impl<'db> RuntimeObj<'db> for StringTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::String(*self)
  }
}

#[tracked]
pub struct StringObj<'db> {
  #[returns(deref)]
  pub value: String,
}

impl<'db> RuntimeObj<'db> for StringObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    StringTyp::new(db).into()
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    self.value(db).to_string()
  }
}
