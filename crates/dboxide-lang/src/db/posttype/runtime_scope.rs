use std::collections::BTreeMap;

use crate::db::types::Obj;

// runtime scope for expression evaluation, carrying bindings from closures and element contexts
#[derive(Clone)]
pub struct RuntimeScope<'db> {
  bindings: BTreeMap<String, Obj<'db>>,
  parent: Option<Box<RuntimeScope<'db>>>,
}

impl<'db> RuntimeScope<'db> {
  pub fn empty() -> Self {
    RuntimeScope {
      bindings: BTreeMap::new(),
      parent: None,
    }
  }

  pub fn with_bindings(bindings: BTreeMap<String, Obj<'db>>) -> Self {
    RuntimeScope {
      bindings,
      parent: None,
    }
  }

  // create a child scope inheriting from this one
  pub fn child(&self, bindings: BTreeMap<String, Obj<'db>>) -> Self {
    RuntimeScope {
      bindings,
      parent: Some(Box::new(self.clone())),
    }
  }

  // create a scope for element context with self and super bindings
  pub fn for_element(self_obj: Obj<'db>, super_obj: Option<Obj<'db>>) -> Self {
    let mut bindings = BTreeMap::new();
    bindings.insert("self".to_string(), self_obj);
    if let Some(sup) = super_obj {
      bindings.insert("super".to_string(), sup);
    }
    RuntimeScope {
      bindings,
      parent: None,
    }
  }

  pub fn lookup(&self, name: &str) -> Option<Obj<'db>> {
    if let Some(obj) = self.bindings.get(name) {
      return Some(*obj);
    }
    if let Some(parent) = &self.parent {
      return parent.lookup(name);
    }
    None
  }
}
