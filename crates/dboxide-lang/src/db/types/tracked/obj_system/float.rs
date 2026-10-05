use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

#[tracked]
pub struct FloatTyp<'db> {}

impl<'db> StaticTyp<'db> for FloatTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "float".to_string()
  }
}

impl<'db> RuntimeObj<'db> for FloatTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Float(*self)
  }
}

/// Stored as string since f64 is not Eq/Hash
#[tracked]
pub struct FloatObj<'db> {
  #[returns(deref)]
  pub value: String,
}

impl<'db> RuntimeObj<'db> for FloatObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    FloatTyp::new(db).into()
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    self.value(db).to_string()
  }
}
