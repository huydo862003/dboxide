mod collection;
mod diagnostics;
mod imports;
mod reuse;

use std::collections::HashMap;

use itertools::Either;
use salsa::Setter;

use crate::db::DboxideDatabase;
use crate::db::pretype_analyze::pretype_get_scope_members;
use crate::db::test_utils::*;
use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind, VirtualModuleKind};
use crate::db::types::{File, StaticScope, StaticScopeKind};

fn get_file_scope_members<'db>(db: &'db DboxideDatabase, file: File) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  pretype_get_scope_members(db, scope).clone()
}
