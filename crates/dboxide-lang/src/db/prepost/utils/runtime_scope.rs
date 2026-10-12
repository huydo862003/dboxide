use std::collections::BTreeMap;

use salsa::{Database, tracked};

use crate::db::types::Obj;

#[tracked]
pub struct RuntimeScope<'db> {
  #[returns(ref)]
  pub bindings: BTreeMap<String, Obj<'db>>,
  #[returns(copy)]
  pub parent: Option<RuntimeScope<'db>>,
}

impl<'db> RuntimeScope<'db> {
  pub fn empty(db: &'db dyn Database) -> Self {
    RuntimeScope::new(db, BTreeMap::new(), None)
  }

  pub fn with_bindings(db: &'db dyn Database, bindings: BTreeMap<String, Obj<'db>>) -> Self {
    RuntimeScope::new(db, bindings, None)
  }

  pub fn child(self, db: &'db dyn Database, bindings: BTreeMap<String, Obj<'db>>) -> Self {
    RuntimeScope::new(db, bindings, Some(self))
  }

  pub fn for_element(
    db: &'db dyn Database,
    self_obj: Obj<'db>,
    super_obj: Option<Obj<'db>>,
  ) -> Self {
    let mut bindings = BTreeMap::new();
    bindings.insert("self".to_string(), self_obj);
    if let Some(sup) = super_obj {
      bindings.insert("super".to_string(), sup);
    }
    RuntimeScope::new(db, bindings, None)
  }

  pub fn lookup(self, db: &'db dyn Database, name: &str) -> Option<Obj<'db>> {
    if let Some(obj) = self.bindings(db).get(name) {
      return Some(*obj);
    }
    self.parent(db)?.lookup(db, name)
  }
}
