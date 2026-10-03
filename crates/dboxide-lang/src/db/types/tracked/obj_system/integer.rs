use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

#[tracked]
pub struct IntegerTyp<'db> {}

impl<'db> StaticTyp<'db> for IntegerTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "integer".to_string()
  }
}

impl<'db> RuntimeObj<'db> for IntegerTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Integer(*self)
  }
}

#[tracked]
pub struct IntegerObj<'db> {
  #[returns(copy)]
  pub value: i64,
}

impl<'db> RuntimeObj<'db> for IntegerObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    IntegerTyp::new(db).into()
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    self.value(db).to_string()
  }
}
