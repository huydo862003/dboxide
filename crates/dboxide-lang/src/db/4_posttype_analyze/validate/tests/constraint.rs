use super::*;
use crate::db::posttype_analyze::posttype_get_scope_members;
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{StaticScope, StaticScopeKind};

#[test]
fn constraint_definition_resolves_column_typ() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  null bool? [setting]
  pk bool? [setting]

  constraints {
    pk_not_null: () => !pk
  }
}
"#,
    metatype(),
  );

  let scope = StaticScope::new(&db, StaticScopeKind::UserModule(file));
  let members = posttype_get_scope_members(&db, scope).clone();
  let symbol = get_first_symbol(&members, "Column");
  assert!(matches!(symbol.kind(&db), SymbolKind::UserTyp(..)));
}
