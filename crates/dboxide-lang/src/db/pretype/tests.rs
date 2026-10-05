use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use itertools::Either;
use salsa::{Database, Setter};

use crate::db::DboxideDatabase;
use crate::db::types::interned::symbol::{SymbolKind, VirtualModuleKind};
use crate::db::types::{File, StaticScope, StaticScopeKind, Symbol};
use crate::{FileHandle, Typ};

use crate::db::types::interned::symbol::StaticScopeMembers;

use super::pretype_evaluate::pretype_evaluate_typ_symbol::pretype_evaluate_typ_symbol;
use super::pretype_get_file_usable_symbols::pretype_get_file_usable_symbols;
use super::pretype_get_scope_members::pretype_get_scope_members;
use super::pretype_get_symbol_namespace::pretype_get_symbol_namespace;
use crate::OnUsePolicy;
use crate::db::types::{Diagnostics, Obj, StaticTyp};
use crate::diagnostics::Diagnostic;

fn create_file(
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

fn get_file_scope_members<'db>(db: &'db DboxideDatabase, file: File) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  pretype_get_scope_members(db, scope).clone()
}

fn assert_contains_symbol(members: &StaticScopeMembers, name: &str) {
  assert!(
    members.lookup_by_name(name).next().is_some(),
    "expected symbol '{name}' in scope"
  );
}

fn assert_missing_symbol(members: &StaticScopeMembers, name: &str) {
  assert!(
    members.lookup_by_name(name).next().is_none(),
    "expected no symbol '{name}' in scope"
  );
}

fn get_first_symbol<'db>(members: &StaticScopeMembers<'db>, name: &str) -> Symbol<'db> {
  *members
    .lookup_by_name(name)
    .next()
    .map(|(_, symbol)| symbol)
    .unwrap_or_else(|| panic!("no symbol '{name}' in scope"))
}

fn get_schema_namespace<'db>(
  db: &'db DboxideDatabase,
  file: File,
  name: &str,
) -> StaticScopeMembers<'db> {
  let members = get_file_scope_members(db, file);
  let schema_symbol = get_first_symbol(&members, name);
  pretype_get_symbol_namespace(db, schema_symbol).clone()
}

// Pretype does NOT collect elements, only types, functions, operators, schemas

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
  let members = get_file_scope_members(&db, file);
  assert_missing_symbol(&members, "users");
  assert_missing_symbol(&members, "posts");
}

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
  let members = get_file_scope_members(&db, file);
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
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "Alias");
  assert!(matches!(
    get_first_symbol(&members, "Alias").kind(&db),
    SymbolKind::UserTyp(..)
  ));
}

#[test]
fn collect_function_declarations() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "fn validate(): bool {}
fn compute(x: int): int {}",
    HashMap::new(),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "validate");
  assert_contains_symbol(&members, "compute");
  assert!(matches!(
    get_first_symbol(&members, "validate").kind(&db),
    SymbolKind::UserFunction(..)
  ));
}

#[test]
fn collect_operator_declarations() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "fn operator >(a: int, b: int): bool {}",
    HashMap::new(),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, ">");
  assert!(matches!(
    get_first_symbol(&members, ">").kind(&db),
    SymbolKind::UserOperator(..)
  ));
}

#[test]
fn element_aliases_not_collected_in_pretype() {
  let db = DboxideDatabase::default();
  let file = create_file(&db, "test.dbml", "Table users as u {}", HashMap::new());
  let members = get_file_scope_members(&db, file);
  assert_missing_symbol(&members, "users");
  assert_missing_symbol(&members, "u");
}

#[test]
fn schema_symbols_discovered_from_elements() {
  let db = DboxideDatabase::default();
  // auth.Table triggers schema "auth" discovery even though Table is an element
  let file = create_file(
    &db,
    "test.dbml",
    "auth.Table users {
  id integer
}",
    HashMap::new(),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "auth");
  assert_contains_symbol(&members, "public");
  assert!(matches!(
    get_first_symbol(&members, "auth").kind(&db),
    SymbolKind::UserSchema(..)
  ));
  // Element itself not in file scope (pretype)
  assert_missing_symbol(&members, "users");
}

#[test]
fn schema_namespace_contains_types_not_elements() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "auth.type Foo {}
auth.type Bar {}
auth.Table users {}
type Baz {}",
    HashMap::new(),
  );
  let auth_namespace = get_schema_namespace(&db, file, "auth");
  // Types in schema are visible
  assert_contains_symbol(&auth_namespace, "Foo");
  assert_contains_symbol(&auth_namespace, "Bar");
  // Elements in schema are NOT visible in pretype
  assert_missing_symbol(&auth_namespace, "users");
  // Types in public schema are not in auth namespace
  assert_missing_symbol(&auth_namespace, "Baz");
}

#[test]
fn nested_schema_namespace() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    "auth.sub.type Inner {}
auth.type Outer {}",
    HashMap::new(),
  );
  let auth_namespace = get_schema_namespace(&db, file, "auth");
  assert_contains_symbol(&auth_namespace, "Outer");
  assert_contains_symbol(&auth_namespace, "sub");
  assert_missing_symbol(&auth_namespace, "Inner");

  let sub_symbol = get_first_symbol(&auth_namespace, "sub");
  let sub_namespace = pretype_get_symbol_namespace(&db, sub_symbol).clone();
  assert_contains_symbol(&sub_namespace, "Inner");
}

#[test]
fn use_wildcard_imports_types_and_functions() {
  let db = DboxideDatabase::default();
  let dep = create_file(
    &db,
    "dep.dbml",
    "type MyTyp {}
fn validate(): bool {}",
    HashMap::new(),
  );
  let file = create_file(
    &db,
    "main.dbml",
    "use * from './dep'",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "MyTyp");
  assert_contains_symbol(&members, "validate");
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
  let members = get_file_scope_members(&db, file);
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
  let members = get_file_scope_members(&db, file);
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
  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "F");
  assert_missing_symbol(&members, "Foo");
  assert_eq!(get_first_symbol(&members, "F").name(&db), "F");
}

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
  let members = get_file_scope_members(&db, file_c);
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
  let usable = pretype_get_file_usable_symbols(&db, dep_b, true);
  assert_missing_symbol(usable, "Foo");
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
  let members = get_file_scope_members(&db, file);
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
  let members = get_file_scope_members(&db, file_a);
  assert_contains_symbol(&members, "Foo");
  assert_contains_symbol(&members, "Bar");
}

#[test]
fn schema_members_are_per_file() {
  let db = DboxideDatabase::default();
  let dep = create_file(&db, "dep.dbml", "auth.type Dep {}", HashMap::new());
  let file = create_file(
    &db,
    "main.dbml",
    "use * from './dep'
auth.type Own {}",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let auth_namespace = get_schema_namespace(&db, file, "auth");
  // Own file's types are visible
  assert_contains_symbol(&auth_namespace, "Own");
  // Dep's types are NOT visible in pretype (import processing is posttype)
  assert_missing_symbol(&auth_namespace, "Dep");
}

#[test]
fn schema_names_are_global() {
  let db = DboxideDatabase::default();
  // dep defines an "auth" schema via auth.Table
  let dep = create_file(&db, "dep.dbml", "auth.Table roles {}", HashMap::new());
  // main imports dep but doesn't have any auth.* declarations
  let file = create_file(
    &db,
    "main.dbml",
    "use * from './dep'",
    HashMap::from([("./dep".to_string(), Either::Right(dep))]),
  );
  let members = get_file_scope_members(&db, file);
  // Schema name "auth" is visible globally even though main doesn't declare auth.* elements
  assert_contains_symbol(&members, "auth");
}

#[test]
fn evaluate_type_element_symbol() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "schema.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  alias qualified_declaration[Table] [label alias]
  pk bool [setting]
  headercolor string? [attribute]

  type Column [field] {
    col_name string
  }

  type Index [element] {
    col_ref string
  }

  namespace {
    name
    get all_columns(): string {
      ""
    }
  }

  fn count(): integer {}
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  assert_contains_symbol(&members, "Table");
  let symbol = get_first_symbol(&members, "Table");
  let obj = pretype_evaluate_typ_symbol(&db, symbol);
  assert!(obj.is_some());
  let Obj::Typ(typ) = obj.unwrap() else {
    panic!("expected Obj::Typ");
  };
  let Typ::Element(element) = typ else {
    panic!("expected Typ::Element");
  };

  assert_eq!(element.name(&db), "Table");

  // Verify labels
  let labels = element.labels(&db);
  assert_eq!(labels.len(), 2);
  let name_label = labels.iter().find(|l| l.field_name == "name").unwrap();
  assert!(!name_label.is_alias);
  assert!(name_label.unique);
  assert!(!name_label.exposes_namespace);

  let alias_label = labels.iter().find(|l| l.field_name == "alias").unwrap();
  assert!(alias_label.is_alias);
  assert!(!alias_label.unique);
  assert!(alias_label.exposes_namespace);

  // Verify settings
  assert!(element.settings(&db).contains_key("pk"));

  // Verify attributes
  assert!(element.attributes(&db).contains_key("headercolor"));

  // Verify fields (@ access has all fields including attributes)
  assert!(element.fields(&db).contains_key("name"));
  assert!(element.fields(&db).contains_key("alias"));
  assert!(element.fields(&db).contains_key("pk"));
  assert!(element.fields(&db).contains_key("headercolor"));

  // Verify nested field subtypes and element subtypes
  assert!(element.field_subtypes(&db).contains_key("Column"));
  assert!(!element.field_subtypes(&db).contains_key("Index"));
  assert!(element.element_subtypes(&db).contains_key("Index"));
  assert!(!element.element_subtypes(&db).contains_key("Column"));

  // Verify namespace
  let namespace = element.namespace(&db);
  assert!(namespace.contains_key("name"));
  assert!(namespace.contains_key("all_columns"));

  // Verify vtable (getter and fn)
  let vtable = element.vtable(&db);
  assert!(vtable.contains_key("all_columns"));
  assert!(vtable.contains_key("count"));

  // Verify static lookup for methods and fields
  assert!(element.lookup_field_typ(&db, "name").is_some());
  assert!(element.lookup_field_typ(&db, "all_columns").is_some());
  assert!(element.lookup_field_typ(&db, "count").is_some());

  // Verify static lookup for namespace members
  assert!(element.lookup_namespace_member(&db, "name").is_some());
  assert!(
    element
      .lookup_namespace_member(&db, "all_columns")
      .is_some()
  );
  assert!(
    element
      .lookup_namespace_member(&db, "nonexistent")
      .is_none()
  );

  // Verify definition struct accessors
  let field_definitions = element.field_definitions(&db);
  assert!(field_definitions.iter().any(|f| f.name == "name"));
  assert!(field_definitions.iter().any(|f| f.name == "headercolor"));

  let attr_definitions = element.attribute_definitions(&db);
  assert!(attr_definitions.iter().any(|a| a.name == "headercolor"));

  let setting_definitions = element.setting_definitions(&db);
  assert!(setting_definitions.iter().any(|s| s.name == "pk"));

  let field_sub_definitions = element.field_subtype_definitions(&db);
  assert!(field_sub_definitions.iter().any(|fst| fst.name == "Column"));
  assert!(!field_sub_definitions.iter().any(|fst| fst.name == "Index"));

  let elem_sub_definitions = element.element_subtype_definitions(&db);
  assert!(elem_sub_definitions.iter().any(|es| es.name == "Index"));
  assert!(!elem_sub_definitions.iter().any(|es| es.name == "Column"));

  let sub_elem_definitions = element.sub_element_definitions(&db);
  assert!(sub_elem_definitions.iter().any(|se| se.name == "Index"));
  assert!(!sub_elem_definitions.iter().any(|se| se.name == "Column"));
}

#[test]
fn declaration_field_without_label_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid_decl.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Table");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    crate::diagnostics::Diagnostic::InvalidDeclarationFieldLabel { field_name, .. } if field_name == "name"
  )));
}

#[test]
fn non_contiguous_arg_index_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid_args.dbml",
    r#"
use * from 'metatype'

type RefBinary [field] {
  from string [arg: 0]
  to string [arg: 2]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "RefBinary");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    crate::diagnostics::Diagnostic::NonContiguousFieldArgIndex {
      field_name,
      found_index: 2,
      expected_index: 1,
      ..
    } if field_name == "to"
  )));
}

#[test]
fn evaluate_element_type_with_constraints() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "constraints.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  null bool? [setting]
  not_null bool? [setting]

  constraints {
    nullability: () => null ^ not_null
  }
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let Some(Obj::Typ(Typ::Element(elem))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  let constraint_defs = elem.constraint_definitions(&db);
  assert_eq!(constraint_defs.len(), 1);
  assert_eq!(constraint_defs[0].name, "nullability");

  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty());
}

#[test]
fn valid_label_with_unique_emits_no_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Table");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty());
}

#[test]
fn valid_csv_with_arg_emits_no_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Row [field] {
  values string [arg: 0, csv]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Row");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn setting_with_key_is_valid() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  refs string [setting: reference]
  deps string [setting: dep]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");

  let Some(Obj::Typ(Typ::Element(elem))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  assert!(elem.settings(&db).contains_key("refs"));
  assert!(elem.settings(&db).contains_key("deps"));

  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn conflicting_label_annotations_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, label alias]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Table");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    crate::diagnostics::Diagnostic::ConflictingLabelAnnotations { field_name, .. } if field_name == "name"
  )));
}

#[test]
fn unique_without_label_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  name string [unique]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    crate::diagnostics::Diagnostic::UniqueRequiresLabel { field_name, .. } if field_name == "name"
  )));
}

#[test]
fn csv_without_arg_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type Row [field] {
  values string [csv]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Row");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    crate::diagnostics::Diagnostic::CsvRequiresArg { field_name, .. } if field_name == "values"
  )));
}

#[test]
fn constraint_without_closure_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid_constraint.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  null bool? [setting]

  constraints {
    invalid_rule: true
  }
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    crate::diagnostics::Diagnostic::InvalidConstraintDefinition { constraint_name, .. } if constraint_name == "invalid_rule"
  )));
}

#[test]
fn variadic_arg_stops_contiguous_validation() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type TableGroup [element] {
  name string [arg: 0]
  tables string [arg: 1..]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "TableGroup");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn arg_after_variadic_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type TableGroup [element] {
  name string [arg: 0]
  tables string [arg: 1..]
  extra string [arg: 2]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "TableGroup");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    Diagnostic::ArgAfterVariadicArg { field_name, .. } if field_name == "extra"
  )));
}

#[test]
fn attribute_field_appears_in_attributes() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  col_type string [arg: 0, attribute]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let Some(Obj::Typ(Typ::Element(elem))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  assert!(
    elem.attributes(&db).contains_key("col_type"),
    "expected 'col_type' in attributes"
  );
  assert!(
    elem.fields(&db).contains_key("col_type"),
    "expected 'col_type' in fields"
  );
  assert!(
    !elem.settings(&db).contains_key("col_type"),
    "expected 'col_type' NOT in settings"
  );

  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn attribute_and_setting_coexist() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  note string? [setting, attribute]
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Table");
  let Some(Obj::Typ(Typ::Element(elem))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  assert!(
    elem.attributes(&db).contains_key("note"),
    "expected 'note' in attributes"
  );
  assert!(
    elem.settings(&db).contains_key("note"),
    "expected 'note' in settings"
  );

  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn colon_syntax_in_type_body_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type Project [element] {
  note: string?
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Project");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    Diagnostic::ColonDeclarationInTypeBody { field_name, .. } if field_name == "note"
  )));
}

#[test]
fn invalid_field_type_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  name 42
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    Diagnostic::InvalidFieldType { field_name, .. } if field_name == "name"
  )));
}

#[test]
fn rest_attribute_bracket_syntax_registers_field() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Index [element] {
  [columns] string
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Index");
  let Some(Obj::Typ(Typ::Element(elem))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  assert!(
    elem.fields(&db).contains_key("columns"),
    "expected 'columns' in fields"
  );

  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn rest_attribute_bracket_syntax_malformed_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type Index [element] {
  [a, b] string
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Index");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    crate::diagnostics::Diagnostic::MalformedRestAttribute { .. }
  )));
}

#[test]
fn member_function_appears_in_vtable() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  fn is_nullable(): bool {}
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  assert!(
    element.vtable(&db).contains_key("is_nullable"),
    "expected 'is_nullable' in vtable"
  );

  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn namespace_block_exposes_field() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  name string

  namespace {
    name
  }
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  assert!(
    element.namespace(&db).contains_key("name"),
    "expected 'name' in namespace"
  );

  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn on_use_expand_policy_is_set() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type TableGroup [element, on use: expand] {
  name string
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "TableGroup");
  let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  assert_eq!(element.on_use(&db), OnUsePolicy::Expand);
}

#[test]
fn unrecognized_on_use_value_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type TableGroup [element, on use: inline] {
  name string
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "TableGroup");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    Diagnostic::UnrecognizedOnUseValue { value, .. } if value == "inline"
  )));
}

#[test]
fn rest_field_invalid_type_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type Index [element] {
  [columns] 42
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Index");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    Diagnostic::InvalidFieldType { field_name, .. } if field_name == "columns"
  )));
}

#[test]
fn getter_in_namespace_block_appears_in_vtable_and_namespace() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  namespace {
    get display_name(): string {}
  }
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  assert!(
    element.namespace(&db).contains_key("display_name"),
    "expected 'display_name' in namespace"
  );
  assert!(
    element.vtable(&db).contains_key("display_name"),
    "expected 'display_name' in vtable"
  );

  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn lookup_namespace_member_typ_resolves_eager_type() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  name string

  namespace {
    name
  }
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  let resolved = element.lookup_namespace_member_typ(&db, "name");
  assert!(
    resolved.is_some(),
    "expected 'name' namespace member to resolve"
  );
  assert!(
    matches!(resolved.unwrap(), Typ::String(_)),
    "expected 'name' to resolve to string"
  );

  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.is_empty(), "unexpected diagnostics");
}

#[test]
fn lookup_constraint_finds_constraint_by_name() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "valid.dbml",
    r#"
use * from 'metatype'

type Project [element] {
  constraints {
    no_nulls: () => true
  }
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Project");
  let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
    panic!("expected Typ::Element");
  };

  let constraint = element.lookup_constraint(&db, "no_nulls");
  assert!(
    constraint.is_some(),
    "expected 'no_nulls' constraint to be found"
  );
  assert_eq!(constraint.unwrap().name, "no_nulls");

  assert!(element.lookup_constraint(&db, "nonexistent").is_none());
}

#[test]
fn unknown_namespace_export_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  name string

  namespace {
    nonexistent
  }
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    Diagnostic::UnknownNamespaceExport { field_name, .. } if field_name == "nonexistent"
  )));
}

#[test]
fn colon_syntax_in_namespace_block_emits_diagnostic() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "invalid.dbml",
    r#"
use * from 'metatype'

type Column [field] {
  namespace {
    name: string
  }
}
"#,
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )]),
  );

  let members = get_file_scope_members(&db, file);
  let symbol = get_first_symbol(&members, "Column");
  let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
  assert!(diagnostics.iter().any(|d| matches!(
    &d.0,
    Diagnostic::ColonDeclarationInNamespaceBlock { field_name, .. } if field_name == "name"
  )));
}
