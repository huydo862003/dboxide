use std::collections::BTreeMap;

use salsa::Database;

use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::typ_param::TypParams;

use super::{FuncObj, FuncTyp, Typ};

pub trait StaticTyp<'db> {
  fn display_name(&self, db: &'db dyn Database) -> String;

  fn arity(&self, _db: &'db dyn Database) -> usize {
    0
  }

  fn parent_typ(&self, _db: &'db dyn Database) -> Option<Typ<'db>> {
    None
  }

  fn is_typ(&self, _db: &'db dyn Database) -> bool {
    false
  }

  fn runtime_typ(&self, _db: &'db dyn Database) -> Option<Typ<'db>> {
    None
  }

  fn typ_params(&self, _db: &'db dyn Database) -> Option<TypParams<'db>> {
    None
  }

  fn instantiate(&self, _db: &'db dyn Database, _args: Vec<LazyTyp<'db>>) -> Option<Typ<'db>> {
    None
  }

  fn construct(&self, _db: &'db dyn Database, _args: Vec<Typ<'db>>) -> Option<Typ<'db>> {
    None
  }

  fn get_fields(&self, _db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    BTreeMap::new()
  }

  fn get_namespace_members(&self, _db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    BTreeMap::new()
  }

  fn get_owned_field_typ(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    self.get_fields(db).get(name).copied()
  }

  fn runtime_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, FuncObj<'db>> {
    self
      .parent_typ(db)
      .map(|p| p.runtime_vtable(db))
      .unwrap_or_default()
  }

  fn static_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, Typ<'db>> {
    self
      .parent_typ(db)
      .map(|p| p.static_vtable(db))
      .unwrap_or_default()
  }

  fn lookup_field_typ(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    if let Some(field) = self.get_owned_field_typ(db, name) {
      return Some(field);
    }
    self.static_vtable(db).get(name).map(|t| LazyTyp::eager(*t))
  }

  fn lookup_namespace_member(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    self.get_namespace_members(db).get(name).copied()
  }

  fn index_typ(&self, db: &'db dyn Database) -> Option<FuncTyp<'db>> {
    let lazy = self.lookup_field_typ(db, "[[index]]")?;
    if let Typ::Func(func) = lazy.as_eager()? {
      return Some(func);
    }
    None
  }

  fn call_typ(&self, db: &'db dyn Database) -> Option<FuncTyp<'db>> {
    let lazy = self.lookup_field_typ(db, "[[call]]")?;
    if let Typ::Func(func) = lazy.as_eager()? {
      return Some(func);
    }
    None
  }
}

pub trait RuntimeObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db>;

  fn get_field(&self, _db: &'db dyn Database, _key: &str) -> Option<Typ<'db>> {
    None
  }

  fn get_owned_field(&self, _db: &'db dyn Database, _key: &str) -> Option<super::Obj<'db>> {
    None
  }

  fn lookup_method(&self, db: &'db dyn Database, key: &str) -> Option<FuncObj<'db>> {
    let mut current = Some(self.get_typ(db));
    while let Some(typ) = current {
      if let Some(func) = typ.runtime_vtable(db).get(key) {
        return Some(*func);
      }
      current = typ.parent_typ(db);
    }
    None
  }

  fn lookup_field(&self, db: &'db dyn Database, key: &str) -> Option<super::Obj<'db>> {
    if let Some(field) = self.get_owned_field(db, key) {
      return Some(field);
    }
    self.lookup_method(db, key).map(super::Obj::from)
  }

  fn index(&self, _db: &'db dyn Database, _key: &super::Obj<'db>) -> Option<super::Obj<'db>> {
    None
  }

  fn source_path(&self, _db: &'db dyn Database) -> String {
    String::new()
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    self.source_path(db)
  }
}
