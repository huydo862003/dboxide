use std::collections::BTreeMap;

use salsa::{Database, tracked};

use crate::db::types::Obj;
use crate::db::types::input::File;

#[tracked]
pub struct RuntimeScope<'db> {
  #[returns(copy)]
  pub file: File,
  #[returns(ref)]
  pub bindings: BTreeMap<String, Obj<'db>>,
  #[returns(copy)]
  pub parent: Option<RuntimeScope<'db>>,
}

impl<'db> RuntimeScope<'db> {
  pub fn empty(db: &'db dyn Database, file: File) -> Self {
    RuntimeScope::new(db, file, BTreeMap::new(), None)
  }

  pub fn with_bindings(
    db: &'db dyn Database,
    file: File,
    bindings: BTreeMap<String, Obj<'db>>,
  ) -> Self {
    RuntimeScope::new(db, file, bindings, None)
  }

  pub fn child(self, db: &'db dyn Database, bindings: BTreeMap<String, Obj<'db>>) -> Self {
    RuntimeScope::new(db, self.file(db), bindings, Some(self))
  }

  pub fn for_element(
    db: &'db dyn Database,
    file: File,
    self_obj: Obj<'db>,
    super_obj: Option<Obj<'db>>,
  ) -> Self {
    let mut bindings = BTreeMap::new();
    bindings.insert("self".to_string(), self_obj);
    if let Some(sup) = super_obj {
      bindings.insert("super".to_string(), sup);
    }
    RuntimeScope::new(db, file, bindings, None)
  }

  pub fn lookup(self, db: &'db dyn Database, name: &str) -> Option<Obj<'db>> {
    if let Some(obj) = self.bindings(db).get(name) {
      return Some(*obj);
    }
    self.parent(db)?.lookup(db, name)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::PathBuf;
  use std::time::SystemTime;

  use salsa::Database;

  use super::RuntimeScope;
  use crate::BoolObj;
  use crate::FileHandle;
  use crate::db::DboxideDatabase;
  use crate::db::types::{File, Obj};

  #[test]
  fn runtime_scope_works() {
    #[salsa::tracked]
    fn run_runtime_scope_test(db: &dyn Database) {
      let now = SystemTime::UNIX_EPOCH;
      let file = File::new(
        db,
        PathBuf::from("test.dbml"),
        FileHandle::Content {
          path: PathBuf::from("test.dbml"),
          content: String::new(),
          ctime: now,
          mtime: now,
        },
        now,
        now,
        HashMap::new(),
      );

      let obj: Obj = BoolObj::new(db, true).into();
      let scope =
        RuntimeScope::with_bindings(db, file, [("pk".to_string(), obj)].into_iter().collect());
      assert!(scope.lookup(db, "pk").is_some());
      assert!(scope.lookup(db, "nonexistent").is_none());

      let child = scope.child(
        db,
        [("null".to_string(), BoolObj::new(db, false).into())]
          .into_iter()
          .collect(),
      );
      assert!(child.lookup(db, "pk").is_some());
      assert!(child.lookup(db, "null").is_some());

      let self_obj: Obj = BoolObj::new(db, true).into();
      let scope2 = RuntimeScope::for_element(db, file, self_obj, None);
      assert!(scope2.lookup(db, "self").is_some());
      assert!(scope2.lookup(db, "super").is_none());

      let scope3 = RuntimeScope::for_element(db, file, self_obj, Some(self_obj));
      assert!(scope3.lookup(db, "super").is_some());
    }

    let db = DboxideDatabase::default();
    run_runtime_scope_test(&db);
  }
}
