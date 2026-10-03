use crate::db::pretype_analyze::pretype_get_file_symbols;
use crate::db::types::Diagnostics;
use crate::diagnostics::DiagnosticKind;

use super::*;

#[test]
fn imports_on_use_expand_exposes_namespace_members() {
  let db = DboxideDatabase::default();
  let dep = create_file(
    &db,
    "dep.dbml",
    r#"
use * from 'metatype'

type Types [element, use: expand namespaces] {
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
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "Types");
  assert_contains_symbol(&members, "varchar");
  assert_contains_symbol(&members, "int");
  assert!(matches!(
    get_first_symbol(&members, "varchar").kind(&db),
    SymbolKind::NamespaceMember(..)
  ));
}

#[test]
fn imports_on_use_default_does_not_expand_namespace() {
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
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "Types");
  assert_missing_symbol(&members, "varchar");
}

#[test]
fn imports_wildcard_imports_typs() {
  let db = DboxideDatabase::default();
  let dep = create_file(&db, "dep.dbml", "type MyTyp {}", HashMap::new());
  let file = create_file(
    &db,
    "main.dbml",
    "use * from './dep'",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "MyTyp");
  assert!(matches!(
    get_first_symbol(&members, "MyTyp").kind(&db),
    SymbolKind::Use(..)
  ));
}

#[test]
fn imports_wildcard_does_not_import_elements() {
  let db = DboxideDatabase::default();
  let dep = create_file(&db, "dep.dbml", "Table users {}", HashMap::new());
  let file = create_file(
    &db,
    "main.dbml",
    "use * from './dep'",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = get_file_scope_members(&db, file);
  assert_missing_symbol(&members, "users");
}

#[test]
fn imports_named_imports_specific_symbols() {
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
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "Foo");
  assert_missing_symbol(&members, "Bar");
}

#[test]
fn imports_multiple_named_specifiers() {
  let db = DboxideDatabase::default();
  let dep = create_file(
    &db,
    "dep.dbml",
    "type Foo {}\ntype Bar {}\ntype Baz {}",
    HashMap::new(),
  );
  let file = create_file(
    &db,
    "main.dbml",
    "use {\n  type Foo\n  type Bar\n} from './dep'",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "Foo");
  assert_contains_symbol(&members, "Bar");
  assert_missing_symbol(&members, "Baz");
}

#[test]
fn imports_with_alias_renames_symbol() {
  let db = DboxideDatabase::default();
  let dep = create_file(&db, "dep.dbml", "type Foo {}", HashMap::new());
  let file = create_file(
    &db,
    "main.dbml",
    "use { type Foo as F } from './dep'",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "F");
  assert_missing_symbol(&members, "Foo");
  assert_eq!(get_first_symbol(&members, "F").name(&db), "F");
}

#[test]
fn imports_metatype_wildcard() {
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
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "string");
  assert_contains_symbol(&members, "integer");
  assert_contains_symbol(&members, "bool");
}

#[test]
fn imports_circular_use_does_not_loop() {
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
  let members = pretype_get_scope_members(&db, scope).clone();
  assert_contains_symbol(&members, "Foo");
  assert_contains_symbol(&members, "Bar");
}

#[test]
fn imports_nonexistent_symbol_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let dep = create_file(&db, "dep.dbml", "type Foo {}", HashMap::new());
  let file = create_file(
    &db,
    "main.dbml",
    "use { type NonExistent } from './dep'",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = get_file_scope_members(&db, file);
  assert_missing_symbol(&members, "NonExistent");
  assert_missing_symbol(&members, "Foo");
  let diagnostics = pretype_get_file_symbols::accumulated::<Diagnostics>(&db, file, false);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.0.kind,
    DiagnosticKind::UnresolvedImport { name, .. } if name == "NonExistent"
  )));
}

#[test]
fn imports_wildcard_brings_schema_into_scope() {
  let db = DboxideDatabase::default();
  let dep = create_file(&db, "dep.dbml", "auth.type Role {}", HashMap::new());
  let file = create_file(
    &db,
    "main.dbml",
    "use * from './dep'",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "auth");
}
