mod alias;
mod anonymous;
mod diagnostics;
mod imports;
mod label;

use std::collections::HashMap;

use itertools::Either;

use crate::db::DboxideDatabase;
use crate::db::posttype_analyze::posttype_get_scope_members;
use crate::db::test_utils::*;
use crate::db::types::interned::symbol::VirtualModuleKind;
use crate::db::types::{File, StaticScope, StaticScopeKind};

fn get_file_scope_members<'db>(
  db: &'db DboxideDatabase,
  file: File,
) -> crate::db::types::interned::symbol::StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  posttype_get_scope_members(db, scope).clone()
}

fn metatype() -> HashMap<String, Either<VirtualModuleKind, File>> {
  HashMap::from([(
    "metatype".to_string(),
    Either::Left(VirtualModuleKind::Metatype),
  )])
}
