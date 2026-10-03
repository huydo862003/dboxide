use super::*;
use crate::db::posttype_analyze::posttype_get_symbol_namespace;
use crate::db::types::interned::symbol::SymbolKind;

#[test]
fn imports_pretype_types_remain_visible_in_posttype_scope() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type MyTyp {}
"#,
    metatype(),
  );

  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "MyTyp");
  assert!(matches!(
    get_first_symbol(&members, "MyTyp").kind(&db),
    SymbolKind::UserTyp(..)
  ));
}

#[test]
fn imports_element_instances_from_wildcard_import_visible() {
  let db = DboxideDatabase::default();
  let dep = create_file(
    &db,
    "dep.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
"#,
    metatype(),
  );
  let file = create_file(
    &db,
    "main.dbml",
    "use * from './dep'",
    HashMap::from([
      ("./dep".to_string(), Either::Right(dep)),
      (
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      ),
    ]),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "Table");
  assert_contains_symbol(&members, "users");
}

mod field_line {
  use super::*;

  fn make_table_with_columns(db: &DboxideDatabase) -> File {
    create_file(
      db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  type Column [field] {
    col_name string [arg: 0, label, unique]
  }
}

Table users {
  id
  email
}
"#,
      metatype(),
    )
  }

  #[test]
  fn field_line_first_arg_is_symbol_name() {
    let db = DboxideDatabase::default();
    let file = make_table_with_columns(&db);
    let file_members = get_file_scope_members(&db, file);
    let users_symbol = get_first_symbol(&file_members, "users");
    let namespace = posttype_get_symbol_namespace(&db, users_symbol);
    assert_contains_symbol(&namespace, "id");
    assert_contains_symbol(&namespace, "email");
    assert_missing_symbol(&namespace, "Column");
  }

  #[test]
  fn field_line_type_arg_not_in_namespace() {
    let db = DboxideDatabase::default();
    let file = make_table_with_columns(&db);
    let file_members = get_file_scope_members(&db, file);
    let users_symbol = get_first_symbol(&file_members, "users");
    let namespace = posttype_get_symbol_namespace(&db, users_symbol);
    assert_missing_symbol(&namespace, "Column");
  }
}
