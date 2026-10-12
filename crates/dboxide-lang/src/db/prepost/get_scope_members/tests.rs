use std::collections::HashMap;

use itertools::Either;
use salsa::Setter;

use crate::db::DboxideDatabase;
use crate::db::prepost::get_file_symbols::get_file_symbols_pretype;
use crate::db::prepost::get_scope_members::{
  get_scope_members_posttype, get_scope_members_pretype,
};
use crate::db::test_utils::*;
use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind, VirtualModuleKind};
use crate::db::types::{File, StaticScope, StaticScopeKind};

fn get_file_scope_members_pretype<'db>(
  db: &'db DboxideDatabase,
  file: File,
) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  get_scope_members_pretype(db, scope).clone()
}

fn get_file_scope_members_posttype<'db>(
  db: &'db DboxideDatabase,
  file: File,
) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  get_scope_members_posttype(db, scope).clone()
}

mod collection {
  use super::*;

  #[test]
  fn collect_type_declarations() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      "type MyType {
    field string
  }",
      HashMap::new(),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "MyType");
    assert!(matches!(
      get_first_symbol(&members, "MyType").kind(&db),
      SymbolKind::UserTyp(..)
    ));
  }

  #[test]
  fn collect_equality_declarations() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      "type Alias = string | null",
      HashMap::new(),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "Alias");
    assert!(matches!(
      get_first_symbol(&members, "Alias").kind(&db),
      SymbolKind::UserTyp(..)
    ));
  }

  #[test]
  fn elements_not_collected_in_pretype() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      "Table users {
    id integer
  }
  Table posts {
    id integer
  }",
      HashMap::new(),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_missing_symbol(&members, "users");
    assert_missing_symbol(&members, "posts");
  }

  #[test]
  fn element_aliases_not_collected_in_pretype() {
    let db = DboxideDatabase::default();
    let file = create_file(&db, "test.dbml", "Table users as u {}", HashMap::new());
    let members = get_file_scope_members_pretype(&db, file);
    assert_missing_symbol(&members, "users");
    assert_missing_symbol(&members, "u");
  }

  #[test]
  fn schema_symbols_discovered_from_elements() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      "auth.Table users {
    id integer
  }",
      HashMap::new(),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "auth");
    assert_contains_symbol(&members, "public");
    assert!(matches!(
      get_first_symbol(&members, "auth").kind(&db),
      SymbolKind::UserSchema(..)
    ));
    assert_missing_symbol(&members, "users");
  }
}

mod imports {
  use super::*;

  #[test]
  fn on_use_expand_exposes_namespace_members_into_importing_scope() {
    let db = DboxideDatabase::default();
    let dep = create_file(
      &db,
      "dep.dbml",
      r#"
use * from 'metatype'

type Types [element, on use: expand] {
  namespace {
    get varchar(): string {}
    get int(): integer {}
  }
}
"#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
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
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "Types");
    assert_contains_symbol(&members, "varchar");
    assert_contains_symbol(&members, "int");
    assert!(matches!(
      get_first_symbol(&members, "varchar").kind(&db),
      SymbolKind::NamespaceMember(..)
    ));
  }

  #[test]
  fn on_use_default_does_not_expand_namespace() {
    let db = DboxideDatabase::default();
    let dep = create_file(
      &db,
      "dep.dbml",
      r#"
use * from 'metatype'

type Types [element] {
  namespace {
    get varchar(): string {}
  }
}
"#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
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
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "Types");
    assert_missing_symbol(&members, "varchar");
  }

  #[test]
  fn use_wildcard_imports_types() {
    let db = DboxideDatabase::default();
    let dep = create_file(&db, "dep.dbml", "type MyTyp {}", HashMap::new());
    let file = create_file(
      &db,
      "main.dbml",
      "use * from './dep'",
      HashMap::from([("./dep".to_string(), Either::Right(dep))]),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "MyTyp");
    assert!(matches!(
      get_first_symbol(&members, "MyTyp").kind(&db),
      SymbolKind::Use(..)
    ));
  }

  #[test]
  fn use_wildcard_does_not_import_elements() {
    let db = DboxideDatabase::default();
    let dep = create_file(&db, "dep.dbml", "Table users {}", HashMap::new());
    let file = create_file(
      &db,
      "main.dbml",
      "use * from './dep'",
      HashMap::from([("./dep".to_string(), Either::Right(dep))]),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_missing_symbol(&members, "users");
  }

  #[test]
  fn use_named_imports() {
    let db = DboxideDatabase::default();
    let dep = create_file(
      &db,
      "dep.dbml",
      "type Foo {}
  type Bar {}",
      HashMap::new(),
    );
    let file = create_file(
      &db,
      "main.dbml",
      "use { type Foo } from './dep'",
      HashMap::from([("./dep".to_string(), Either::Right(dep))]),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "Foo");
    assert_missing_symbol(&members, "Bar");
  }

  #[test]
  fn use_with_alias() {
    let db = DboxideDatabase::default();
    let dep = create_file(&db, "dep.dbml", "type Foo {}", HashMap::new());
    let file = create_file(
      &db,
      "main.dbml",
      "use { type Foo as F } from './dep'",
      HashMap::from([("./dep".to_string(), Either::Right(dep))]),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "F");
    assert_missing_symbol(&members, "Foo");
    assert_eq!(get_first_symbol(&members, "F").name(&db), "F");
  }

  #[test]
  fn use_metatype_wildcard() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      "use * from 'metatype'",
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "string");
    assert_contains_symbol(&members, "integer");
    assert_contains_symbol(&members, "bool");
  }

  #[test]
  fn circular_use_does_not_loop() {
    let mut db = DboxideDatabase::default();
    let file_a = create_file(
      &db,
      "a.dbml",
      "use * from './b'
  type Foo {}",
      HashMap::new(),
    );
    let file_b = create_file(
      &db,
      "b.dbml",
      "use * from './a'
  type Bar {}",
      HashMap::new(),
    );
    file_a
      .set_uses(&mut db)
      .to(HashMap::from([("./b".to_string(), Either::Right(file_b))]));
    file_b
      .set_uses(&mut db)
      .to(HashMap::from([("./a".to_string(), Either::Right(file_a))]));
    let scope = StaticScope::new(&db, StaticScopeKind::UserModule(file_a));
    let members = get_scope_members_pretype(&db, scope).clone();
    assert_contains_symbol(&members, "Foo");
    assert_contains_symbol(&members, "Bar");
  }
}

mod reuse {
  use super::*;

  #[test]
  fn reuse_exports_transitively() {
    let db = DboxideDatabase::default();
    let dep_a = create_file(&db, "a.dbml", "type Foo {}", HashMap::new());
    let dep_b = create_file(
      &db,
      "b.dbml",
      "reuse * from './a'",
      HashMap::from([("./a".to_string(), Either::Right(dep_a))]),
    );
    let file_c = create_file(
      &db,
      "c.dbml",
      "use * from './b'",
      HashMap::from([("./b".to_string(), Either::Right(dep_b))]),
    );
    let members = get_file_scope_members_pretype(&db, file_c);
    assert_contains_symbol(&members, "Foo");
  }

  #[test]
  fn use_does_not_export() {
    let db = DboxideDatabase::default();
    let dep_a = create_file(&db, "a.dbml", "type Foo {}", HashMap::new());
    let dep_b = create_file(
      &db,
      "b.dbml",
      "use * from './a'",
      HashMap::from([("./a".to_string(), Either::Right(dep_a))]),
    );
    let usable = get_file_symbols_pretype(&db, dep_b, true);
    assert_missing_symbol(usable, "Foo");
  }

  #[test]
  fn schema_names_are_global() {
    let db = DboxideDatabase::default();
    let dep = create_file(&db, "dep.dbml", "auth.Table roles {}", HashMap::new());
    let file = create_file(
      &db,
      "main.dbml",
      "use * from './dep'",
      HashMap::from([("./dep".to_string(), Either::Right(dep))]),
    );
    let members = get_file_scope_members_pretype(&db, file);
    assert_contains_symbol(&members, "auth");
  }

  #[test]
  fn reuse_exports_schemas_transitively() {
    let db = DboxideDatabase::default();
    let dep_a = create_file(&db, "a.dbml", "auth.type Role {}", HashMap::new());
    let dep_b = create_file(
      &db,
      "b.dbml",
      "reuse * from './a'",
      HashMap::from([("./a".to_string(), Either::Right(dep_a))]),
    );
    let file_c = create_file(
      &db,
      "c.dbml",
      "use * from './b'",
      HashMap::from([("./b".to_string(), Either::Right(dep_b))]),
    );
    let members = get_file_scope_members_pretype(&db, file_c);
    // Schema 'auth' from dep_a should be transitively visible via reuse in dep_b
    assert_contains_symbol(&members, "auth");
  }
}

mod posttype {
  use super::*;

  #[test]
  fn types_from_pretype_still_visible() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
  use * from 'metatype'

  type MyTyp {}
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members_posttype(&db, file);
    assert_contains_symbol(&members, "MyTyp");
    assert!(matches!(
      get_first_symbol(&members, "MyTyp").kind(&db),
      SymbolKind::UserTyp(..)
    ));
  }

  #[test]
  fn element_instances_from_imported_file_not_visible_posttype() {
    // `use` only imports types/schemas, element instances defined in the dep are NOT re-exported
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
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
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
    let members = get_file_scope_members_posttype(&db, file);
    // The type Table is imported but element instance 'users' is NOT
    assert_contains_symbol(&members, "Table");
    assert_missing_symbol(&members, "users");
  }

  #[test]
  fn unlabeled_reference_element_gets_anonymous_symbol_in_scope() {
    // Elements with reference-typed fields but no label fields are implicitly in scope
    // as anonymous symbols (name starts with "@anon::")
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

type Ref [element] {
  from reference[Table] [arg: 0]
  to reference[Table] [arg: 1]
}

Table users {}
Table orders {}
Ref {}
"#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members_posttype(&db, file);
    assert_contains_symbol(&members, "users");
    assert_contains_symbol(&members, "orders");
    assert!(
      members.iter().any(|(key, _)| key.0.starts_with("@anon::")),
      "expected anonymous symbol for the unlabeled Ref element"
    );
  }

  #[test]
  fn labeled_element_does_not_get_anonymous_symbol() {
    // Labeled elements get their own named symbol, not an anonymous one
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
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members_posttype(&db, file);
    assert_contains_symbol(&members, "users");
    assert!(
      !members.iter().any(|(key, _)| key.0.starts_with("@anon::")),
      "labeled element should not produce an anonymous symbol"
    );
  }
}
