use salsa::interned;

use super::lazy_typ::LazyTyp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Variance {
  Covariant,
  Contravariant,
  Invariant,
}

#[interned]
pub struct TypVariable<'db> {
  #[returns(copy)]
  pub upper_bound: Option<LazyTyp<'db>>,
  #[returns(copy)]
  pub variance: Variance,
}

#[interned]
pub struct TypParams<'db> {
  #[returns(ref)]
  pub params: Vec<TypVariable<'db>>,
  #[returns(ref)]
  pub bindings: Vec<LazyTyp<'db>>,
}

impl<'db> TypParams<'db> {
  pub fn arity(&self, db: &'db dyn salsa::Database) -> usize {
    self
      .params(db)
      .len()
      .saturating_sub(self.bindings(db).len())
  }

  pub fn is_instantiated(&self, db: &'db dyn salsa::Database) -> bool {
    let params = self.params(db);
    let bindings = self.bindings(db);
    !params.is_empty() && params.len() == bindings.len()
  }
}
