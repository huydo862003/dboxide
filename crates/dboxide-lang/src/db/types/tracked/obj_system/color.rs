use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};
use super::{IntegerObj, Obj};

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

#[tracked]
pub struct ColorObj<'db> {
  #[returns(copy)]
  pub r: u8,
  #[returns(copy)]
  pub g: u8,
  #[returns(copy)]
  pub b: u8,
}

impl<'db> RuntimeObj<'db> for ColorObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    ColorTyp::new(db).into()
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    format!("#{:02x}{:02x}{:02x}", self.r(db), self.g(db), self.b(db))
  }

  fn lookup_field(&self, db: &'db dyn Database, key: &str) -> Option<Obj<'db>> {
    match key {
      "r" => Some(IntegerObj::new(db, self.r(db) as i64).into()),
      "g" => Some(IntegerObj::new(db, self.g(db) as i64).into()),
      "b" => Some(IntegerObj::new(db, self.b(db) as i64).into()),
      _ => None,
    }
  }
}
