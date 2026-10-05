use salsa::{Database, tracked};

use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::typ_param::{TypParams, TypVariable, Variance};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// `reference[T]`
#[tracked]
pub struct ReferenceTyp<'db> {
  pub param: Option<LazyTyp<'db>>,
}

impl<'db> StaticTyp<'db> for ReferenceTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "reference".to_string()
  }

  fn arity(&self, db: &'db dyn Database) -> usize {
    if self.param(db).is_some() { 0 } else { 1 }
  }

  fn typ_params(&self, db: &'db dyn Database) -> Option<TypParams<'db>> {
    if self.param(db).is_some() {
      return None;
    }
    Some(TypParams::new(
      db,
      vec![TypVariable::new(db, None, Variance::Covariant)],
      vec![],
    ))
  }

  fn instantiate(&self, db: &'db dyn Database, args: Vec<LazyTyp<'db>>) -> Option<Typ<'db>> {
    if args.len() != 1 {
      return None;
    }
    Some(ReferenceTyp::new(db, Some(args[0])).into())
  }
}

impl<'db> RuntimeObj<'db> for ReferenceTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Reference(*self)
  }
}

/// `qualified_reference[T]`
#[tracked]
pub struct QualifiedReferenceTyp<'db> {
  pub param: Option<LazyTyp<'db>>,
}

impl<'db> StaticTyp<'db> for QualifiedReferenceTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "qualified_reference".to_string()
  }

  fn arity(&self, db: &'db dyn Database) -> usize {
    if self.param(db).is_some() { 0 } else { 1 }
  }

  fn typ_params(&self, db: &'db dyn Database) -> Option<TypParams<'db>> {
    if self.param(db).is_some() {
      return None;
    }
    Some(TypParams::new(
      db,
      vec![TypVariable::new(db, None, Variance::Covariant)],
      vec![],
    ))
  }

  fn instantiate(&self, db: &'db dyn Database, args: Vec<LazyTyp<'db>>) -> Option<Typ<'db>> {
    if args.len() != 1 {
      return None;
    }
    Some(QualifiedReferenceTyp::new(db, Some(args[0])).into())
  }
}

impl<'db> RuntimeObj<'db> for QualifiedReferenceTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::QualifiedReference(*self)
  }
}
