use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

#[tracked]
pub struct BoolTyp<'db> {}

impl<'db> StaticTyp<'db> for BoolTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "bool".to_string()
  }
}

impl<'db> RuntimeObj<'db> for BoolTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Bool(*self)
  }
}

#[tracked]
pub struct BoolObj<'db> {
  #[returns(copy)]
  pub value: bool,
}

impl<'db> RuntimeObj<'db> for BoolObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    BoolTyp::new(db).into()
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    self.value(db).to_string()
  }
}
