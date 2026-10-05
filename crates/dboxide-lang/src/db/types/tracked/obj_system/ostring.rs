use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// Opaque string, passed through verbatim
#[tracked]
pub struct OStringTyp<'db> {}

impl<'db> StaticTyp<'db> for OStringTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "ostring".to_string()
  }
}

impl<'db> RuntimeObj<'db> for OStringTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::OString(*self)
  }
}

#[tracked]
pub struct OStringObj<'db> {
  #[returns(deref)]
  pub value: String,
}

impl<'db> RuntimeObj<'db> for OStringObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    OStringTyp::new(db).into()
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    self.value(db).to_string()
  }
}
