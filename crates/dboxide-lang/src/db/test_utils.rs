use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use itertools::Either;
use salsa::Database;

use crate::FileHandle;
use crate::db::types::File;
use crate::db::types::interned::symbol::{StaticScopeMembers, Symbol, VirtualModuleKind};

pub(crate) fn create_file(
  db: &dyn Database,
  path: &str,
  content: &str,
  uses: HashMap<String, Either<VirtualModuleKind, File>>,
) -> File {
  let now = SystemTime::UNIX_EPOCH;
  File::new(
    db,
    PathBuf::from(path),
    FileHandle::Content {
      path: PathBuf::from(path),
      content: content.to_string(),
      ctime: now,
      mtime: now,
    },
    now,
    now,
    uses,
  )
}

pub(crate) fn assert_contains_symbol(members: &StaticScopeMembers, name: &str) {
  assert!(
    members.lookup_by_name(name).next().is_some(),
    "expected symbol '{name}' in scope"
  );
}

pub(crate) fn assert_missing_symbol(members: &StaticScopeMembers, name: &str) {
  assert!(
    members.lookup_by_name(name).next().is_none(),
    "expected no symbol '{name}' in scope"
  );
}

pub(crate) fn get_first_symbol<'db>(members: &StaticScopeMembers<'db>, name: &str) -> Symbol<'db> {
  *members
    .lookup_by_name(name)
    .next()
    .map(|(_, symbol)| symbol)
    .unwrap_or_else(|| panic!("no symbol '{name}' in scope"))
}
