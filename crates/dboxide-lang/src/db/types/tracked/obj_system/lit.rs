use std::collections::BTreeMap;

use salsa::{Database, tracked};

use crate::db::types::interned::lazy_typ::LazyTyp;

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// Literal type: `"btree"`, `42`, `true`
#[tracked]
pub struct LitTyp<'db> {
  pub value: LitValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LitValue {
  String(String),
  Integer(i64),
  Float(String), // stored as string since f64 is not Eq/Hash
  Bool(bool),
}

impl<'db> LitTyp<'db> {
  pub fn get_underlying_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    match &self.value(db) {
      LitValue::String(_) => super::StringTyp::new(db).into(),
      LitValue::Integer(_) => super::IntegerTyp::new(db).into(),
      LitValue::Float(_) => super::FloatTyp::new(db).into(),
      LitValue::Bool(_) => super::BoolTyp::new(db).into(),
    }
  }
}

impl<'db> StaticTyp<'db> for LitTyp<'db> {
  fn display_name(&self, db: &'db dyn Database) -> String {
    match &self.value(db) {
      LitValue::String(s) => format!("\"{}\"", s),
      LitValue::Integer(n) => n.to_string(),
      LitValue::Float(s) => s.clone(),
      LitValue::Bool(b) => b.to_string(),
    }
  }

  fn runtime_typ(&self, db: &'db dyn Database) -> Option<Typ<'db>> {
    Some(self.get_underlying_typ(db))
  }

  fn get_fields(&self, db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    self.get_underlying_typ(db).get_fields(db)
  }

  fn static_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, Typ<'db>> {
    self.get_underlying_typ(db).static_vtable(db)
  }
}

impl<'db> RuntimeObj<'db> for LitTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Lit(*self)
  }
}
