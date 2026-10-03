use salsa::{Database, tracked};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// ISO 8601 datetime, subtype of string
#[tracked]
pub struct DateTimeTyp<'db> {}

impl<'db> StaticTyp<'db> for DateTimeTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "datetime".to_string()
  }
}

impl<'db> RuntimeObj<'db> for DateTimeTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::DateTime(*self)
  }
}

/// ISO 8601 date, subtype of string
#[tracked]
pub struct DateTyp<'db> {}

impl<'db> StaticTyp<'db> for DateTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "date".to_string()
  }
}

impl<'db> RuntimeObj<'db> for DateTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Date(*self)
  }
}

/// ISO 8601 time, subtype of string
#[tracked]
pub struct TimeTyp<'db> {}

impl<'db> StaticTyp<'db> for TimeTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "time".to_string()
  }
}

impl<'db> RuntimeObj<'db> for TimeTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Time(*self)
  }
}
