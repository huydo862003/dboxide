use salsa::Database;

use crate::db::types::Symbol;
use crate::db::types::tracked::obj_system::Typ;
use crate::pretype_evaluate_typ_symbol;

/// A type reference that may be eagerly resolved or lazily deferred to a symbol
/// Enables forward references and circular type dependencies
#[derive(Clone, Copy, PartialEq, Eq, Hash, salsa::SalsaValue)]
pub enum LazyTyp<'db> {
  Eager(Typ<'db>),
  Lazy(Symbol<'db>),
}

impl<'db> std::fmt::Debug for LazyTyp<'db> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      LazyTyp::Eager(_) => write!(f, "LazyTyp::Eager(..)"),
      LazyTyp::Lazy(_) => write!(f, "LazyTyp::Lazy(..)"),
    }
  }
}

impl<'db> LazyTyp<'db> {
  pub fn eager(typ: Typ<'db>) -> Self {
    LazyTyp::Eager(typ)
  }

  pub fn lazy(symbol: Symbol<'db>) -> Self {
    LazyTyp::Lazy(symbol)
  }

  pub fn as_eager(&self) -> Option<Typ<'db>> {
    match self {
      LazyTyp::Eager(typ) => Some(*typ),
      LazyTyp::Lazy(_) => None,
    }
  }

  pub fn resolve(&self, db: &'db dyn Database) -> Option<Typ<'db>> {
    match self {
      LazyTyp::Eager(typ) => Some(*typ),
      LazyTyp::Lazy(symbol) => {
        pretype_evaluate_typ_symbol(db, *symbol).and_then(|obj| obj.as_typ())
      }
    }
  }
}

impl<'db> From<Typ<'db>> for LazyTyp<'db> {
  fn from(typ: Typ<'db>) -> Self {
    LazyTyp::Eager(typ)
  }
}

impl<'db> From<Symbol<'db>> for LazyTyp<'db> {
  fn from(symbol: Symbol<'db>) -> Self {
    LazyTyp::Lazy(symbol)
  }
}
