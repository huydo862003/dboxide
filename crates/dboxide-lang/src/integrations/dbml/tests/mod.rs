mod bind;
mod interpret;
mod validate;

use std::collections::HashMap;

use itertools::Either;

use crate::db::test_utils::*;
use crate::diagnostics::DiagnosticKind;
use crate::{
  DboxideDatabase, Diagnostics, File, StaticScope, StaticScopeKind, StaticScopeMembers, SymbolKind,
  VirtualModuleKind, posttype_get_file_symbols, posttype_get_scope_members,
  pretype_get_file_symbols, pretype_get_scope_members, pretype_get_symbol_namespace,
  validate_file_elements,
};

use super::DBML_PRELUDE;

type Uses = HashMap<String, Either<VirtualModuleKind, File>>;

fn metatype() -> Uses {
  HashMap::from([(
    "metatype".to_string(),
    Either::Left(VirtualModuleKind::Metatype),
  )])
}

fn setup(db: &DboxideDatabase) -> File {
  create_file(db, "prelude.dbml", DBML_PRELUDE, metatype())
}

fn dbml(db: &DboxideDatabase, prelude: File, path: &str, content: &str, extra_uses: Uses) -> File {
  let mut uses = HashMap::from([
    (
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    ),
    ("./prelude".to_string(), Either::Right(prelude)),
  ]);
  uses.extend(extra_uses);
  create_file(
    db,
    path,
    &format!("use * from './prelude'\n{content}"),
    uses,
  )
}

fn pretype_members<'db>(db: &'db DboxideDatabase, file: File) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  pretype_get_scope_members(db, scope).clone()
}

fn posttype_members<'db>(db: &'db DboxideDatabase, file: File) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  posttype_get_scope_members(db, scope).clone()
}

fn assert_no_diagnostics(db: &DboxideDatabase, file: File) {
  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(db, file);
  assert!(
    diagnostics.is_empty(),
    "expected no diagnostics, got: {:?}",
    diagnostics
      .iter()
      .map(|diagnostic| diagnostic.0.message())
      .collect::<Vec<_>>()
  );
}

fn assert_has_diagnostic(
  db: &DboxideDatabase,
  file: File,
  predicate: impl Fn(&DiagnosticKind) -> bool,
) {
  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(db, file);
  assert!(
    diagnostics
      .iter()
      .any(|diagnostic| predicate(&diagnostic.0.kind)),
    "expected matching diagnostic, got: {:?}",
    diagnostics
      .iter()
      .map(|diagnostic| diagnostic.0.message())
      .collect::<Vec<_>>()
  );
}

fn assert_no_matching_diagnostic(
  db: &DboxideDatabase,
  file: File,
  predicate: impl Fn(&DiagnosticKind) -> bool,
) {
  let diagnostics = validate_file_elements::accumulated::<Diagnostics>(db, file);
  assert!(
    !diagnostics
      .iter()
      .any(|diagnostic| predicate(&diagnostic.0.kind)),
    "expected no matching diagnostic, got: {:?}",
    diagnostics
      .iter()
      .map(|diagnostic| diagnostic.0.message())
      .collect::<Vec<_>>()
  );
}
