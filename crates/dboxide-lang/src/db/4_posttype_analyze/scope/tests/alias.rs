use super::*;
use crate::db::posttype_analyze::posttype_get_scope_members;
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{Diagnostics, StaticScope, StaticScopeKind};
use crate::diagnostics::DiagnosticKind;

#[test]
fn alias_element_declares_alias_symbol() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  alias qualified_declaration[Table] [label alias]
}

Table users as u {}
"#,
    metatype(),
  );

  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "users");
  assert_contains_symbol(&members, "u");
  assert!(matches!(
    get_first_symbol(&members, "users").kind(&db),
    SymbolKind::UserElement(..)
  ));
  assert!(matches!(
    get_first_symbol(&members, "u").kind(&db),
    SymbolKind::ElementAlias(..)
  ));
}

#[test]
fn alias_without_primary_name_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  alias qualified_declaration[Table] [label alias]
}

Table as u {}
"#,
    metatype(),
  );

  let scope = StaticScope::new(&db, StaticScopeKind::UserModule(file));
  let diagnostics = posttype_get_scope_members::accumulated::<Diagnostics>(&db, scope);
  assert!(
    diagnostics
      .iter()
      .any(|diagnostic| matches!(diagnostic.0.kind, DiagnosticKind::AliasMissingPrimaryName)),
    "expected AliasMissingPrimaryName diagnostic"
  );
}

#[test]
fn alias_on_typ_without_alias_label_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users as u {}
"#,
    metatype(),
  );

  let scope = StaticScope::new(&db, StaticScopeKind::UserModule(file));
  let diagnostics = posttype_get_scope_members::accumulated::<Diagnostics>(&db, scope);
  assert!(
    diagnostics.iter().any(|diagnostic| matches!(
      diagnostic.0.kind,
      DiagnosticKind::AliasNotSupportedByTyp { .. }
    )),
    "expected AliasNotSupportedByTyp diagnostic"
  );
}
