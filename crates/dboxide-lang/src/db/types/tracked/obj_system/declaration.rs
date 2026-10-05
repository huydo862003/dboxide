use salsa::{Database, tracked};

use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::typ_param::{TypParams, TypVariable, Variance};

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

/// `declaration[T]`
#[tracked]
pub struct DeclarationTyp<'db> {
  pub param: Option<LazyTyp<'db>>,
}

impl<'db> StaticTyp<'db> for DeclarationTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "declaration".to_string()
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
      vec![TypVariable::new(db, None, Variance::Invariant)],
      vec![],
    ))
  }

  fn instantiate(&self, db: &'db dyn Database, args: Vec<LazyTyp<'db>>) -> Option<Typ<'db>> {
    if args.len() != 1 {
      return None;
    }
    Some(DeclarationTyp::new(db, Some(args[0])).into())
  }
}

impl<'db> RuntimeObj<'db> for DeclarationTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Declaration(*self)
  }
}

/// `qualified_declaration[T]`
#[tracked]
pub struct QualifiedDeclarationTyp<'db> {
  pub param: Option<LazyTyp<'db>>,
}

impl<'db> StaticTyp<'db> for QualifiedDeclarationTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "qualified_declaration".to_string()
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
      vec![TypVariable::new(db, None, Variance::Invariant)],
      vec![],
    ))
  }

  fn instantiate(&self, db: &'db dyn Database, args: Vec<LazyTyp<'db>>) -> Option<Typ<'db>> {
    if args.len() != 1 {
      return None;
    }
    Some(QualifiedDeclarationTyp::new(db, Some(args[0])).into())
  }
}

impl<'db> RuntimeObj<'db> for QualifiedDeclarationTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::QualifiedDeclaration(*self)
  }
}
