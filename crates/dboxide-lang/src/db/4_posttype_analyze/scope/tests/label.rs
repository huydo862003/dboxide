use super::*;
use crate::db::posttype_analyze::posttype_get_scope_members;
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{Diagnostics, StaticScope, StaticScopeKind};
use crate::diagnostics::DiagnosticKind;

#[test]
fn label_element_declares_symbol_in_scope() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
"#,
    metatype(),
  );

  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "users");
  assert!(matches!(
    get_first_symbol(&members, "users").kind(&db),
    SymbolKind::UserElement(..)
  ));
  assert_contains_symbol(&members, "Table");
}

#[test]
fn label_unlabeled_typ_declares_no_named_symbol() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Note [element] {
  content string [arg: 0]
}

Note { 'hello' }
"#,
    metatype(),
  );

  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "Note");
}

#[test]
fn label_name_on_unlabeled_typ_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Note [element] {
  content string [arg: 0]
}

Note myname { 'hello' }
"#,
    metatype(),
  );

  let scope = StaticScope::new(&db, StaticScopeKind::UserModule(file));
  let diagnostics = posttype_get_scope_members::accumulated::<Diagnostics>(&db, scope);
  assert!(
    diagnostics.iter().any(|diagnostic| matches!(
      diagnostic.0.kind,
      DiagnosticKind::NameNotSupportedByTyp { .. }
    )),
    "expected NameNotSupportedByTyp diagnostic"
  );
}
