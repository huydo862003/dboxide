use std::collections::HashMap;

use itertools::Either;

use crate::db::DboxideDatabase;
use crate::db::prepost::get_scope_members::get_scope_members_pretype;
use crate::db::pretype::evaluate_typ::evaluate_typ_symbol;
use crate::db::test_utils::*;
use crate::db::types::interned::symbol::{StaticScopeMembers, VirtualModuleKind};
use crate::db::types::{Diagnostics, File, Obj, StaticScope, StaticScopeKind, StaticTyp};
use crate::diagnostics::DiagnosticKind;
use crate::{OnUsePolicy, Typ};

fn get_file_scope_members<'db>(db: &'db DboxideDatabase, file: File) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  get_scope_members_pretype(db, scope).clone()
}

mod roles {
  use super::*;

  #[test]
  fn unrecognized_type_role_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Foo [widget] {}
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Foo");
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::UnrecognizedTypSetting { setting } if setting == "widget"
    )));
  }

  #[test]
  fn known_roles_emit_no_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Foo [element] {}
  type Bar [field] {}
  type Baz [element, on use: expand] {}
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    for name in ["Foo", "Bar", "Baz"] {
      let symbol = get_first_symbol(&members, name);
      let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
      assert!(
        !diagnostics.iter().any(|diagnostic| matches!(
          &diagnostic.0.kind,
          DiagnosticKind::UnrecognizedTypSetting { .. }
        )),
        "unexpected UnrecognizedTypSetting diagnostic on {name}"
      );
    }
  }

  #[test]
  fn empty_type_body_has_no_fields() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Foo {}
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Foo");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element_typ.typ_field_lines(&db).is_empty());
    assert!(element_typ.typ_element_labels(&db).is_empty());
    assert!(element_typ.typ_element_settings(&db).is_empty());
    assert!(element_typ.typ_element_attributes(&db).named.is_empty());

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert_eq!(element.typ_settings(&db).on_use, OnUsePolicy::Expand);
  }

  #[test]
  fn element_and_field_together_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Foo [element, field] {}
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Foo");
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics
        .iter()
        .any(|d| matches!(d.0.kind, DiagnosticKind::ConflictingTypRoles)),
      "expected ConflictingTypRoles diagnostic"
    );
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::UnrecognizedOnUseValue { value } if value == "inline"
    )));
  }
}

mod fields {
  use super::*;

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
    let obj = evaluate_typ_symbol(&db, symbol);
    assert!(obj.is_some());
    let Obj::Typ(typ) = obj.unwrap() else {
      panic!("expected Obj::Typ");
    };
    let Typ::Element(element) = typ else {
      panic!("expected Typ::Element");
    };

    assert_eq!(element.name(&db), "Table");

    let labels = element.typ_element_labels(&db);
    assert_eq!(labels.len(), 2);
    let name_label = labels.iter().find(|l| l.field_name == "name").unwrap();
    assert!(!name_label.is_alias);
    assert!(name_label.unique);
    assert!(!name_label.exposes_namespace);

    let alias_label = labels.iter().find(|l| l.field_name == "alias").unwrap();
    assert!(alias_label.is_alias);
    assert!(!alias_label.unique);
    assert!(alias_label.exposes_namespace);

    assert!(element.typ_element_settings(&db).contains_key("pk"));
    assert!(
      element
        .typ_element_attributes(&db)
        .named
        .contains_key("headercolor")
    );
    assert!(element.typ_field_lines(&db).contains_key("name"));
    assert!(element.typ_field_lines(&db).contains_key("alias"));
    assert!(element.typ_field_lines(&db).contains_key("pk"));
    assert!(element.typ_field_lines(&db).contains_key("headercolor"));

    assert!(
      element
        .typ_nested_typs(&db)
        .field_nested_typs
        .contains_key("Column")
    );
    assert!(
      !element
        .typ_nested_typs(&db)
        .field_nested_typs
        .contains_key("Index")
    );
    assert!(
      element
        .typ_nested_typs(&db)
        .element_nested_typs
        .contains_key("Index")
    );
    assert!(
      !element
        .typ_nested_typs(&db)
        .element_nested_typs
        .contains_key("Column")
    );

    let namespace = element.typ_namespace(&db);
    assert!(namespace.contains_key("name"));
    assert!(namespace.contains_key("all_columns"));

    let vtable = element.vtable(&db);
    assert!(vtable.contains_key("all_columns"));
    assert!(vtable.contains_key("count"));

    assert!(element.lookup_field_typ(&db, "name").is_some());
    assert!(element.lookup_field_typ(&db, "all_columns").is_some());
    assert!(element.lookup_field_typ(&db, "count").is_some());

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

    let field_definitions = element.field_definitions(&db);
    assert!(field_definitions.iter().any(|f| f.name == "name"));
    assert!(field_definitions.iter().any(|f| f.name == "headercolor"));

    let attr_definitions = element.attribute_definitions(&db);
    assert!(attr_definitions.iter().any(|a| a.name == "headercolor"));

    let setting_definitions = element.setting_definitions(&db);
    assert!(setting_definitions.iter().any(|s| s.name == "pk"));

    let field_nested_typ_defs = element.field_nested_typ_definitions(&db);
    assert!(field_nested_typ_defs.iter().any(|def| def.name == "Column"));
    assert!(!field_nested_typ_defs.iter().any(|def| def.name == "Index"));

    let element_nested_typ_defs = element.element_nested_typ_definitions(&db);
    assert!(
      element_nested_typ_defs
        .iter()
        .any(|def| def.name == "Index")
    );
    assert!(
      !element_nested_typ_defs
        .iter()
        .any(|def| def.name == "Column")
    );

    let nested_element_defs = element.nested_typ_definitions(&db);
    assert!(nested_element_defs.iter().any(|def| def.name == "Index"));
    assert!(!nested_element_defs.iter().any(|def| def.name == "Column"));
  }

  #[test]
  fn nullable_field_type_resolves_to_sum() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Column [field] {
    name string?
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Column");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let lazy = element_typ.lookup_field_typ(&db, "name");
    assert!(lazy.is_some(), "expected field 'name' to exist");
    let resolved = lazy.unwrap().resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::Sum(_))),
      "expected Sum type for nullable field, got (non-Debug type)"
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn generic_parameterized_field_type_resolves_to_list() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Row [field] {
    tags List[string]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Row");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let lazy = element_typ.lookup_field_typ(&db, "tags");
    assert!(lazy.is_some(), "expected field 'tags' to exist");
    let resolved = lazy.unwrap().resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::List(_))),
      "expected List type for generic field, got (non-Debug type)"
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::InvalidFieldTyp { field_name } if field_name == "name"
    )));
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::ColonDeclarationInTypBody { field_name } if field_name == "note"
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element_typ
        .typ_element_attributes(&db)
        .named
        .contains_key("col_type")
    );
    assert!(element_typ.typ_field_lines(&db).contains_key("col_type"));
    assert!(
      !element_typ
        .typ_element_settings(&db)
        .contains_key("col_type")
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element_typ
        .typ_element_attributes(&db)
        .named
        .contains_key("note")
    );
    assert!(element_typ.typ_element_settings(&db).contains_key("note"));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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

    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element_typ.typ_element_settings(&db).contains_key("refs"));
    assert!(element_typ.typ_element_settings(&db).contains_key("deps"));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element_typ.typ_element_attributes(&db).rest.is_some(),
      "expected rest attribute to be set"
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics
        .iter()
        .any(|d| matches!(&d.0.kind, DiagnosticKind::MalformedRestAttribute))
    );
  }

  #[test]
  fn rest_attribute_duplicate_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Index [element] {
    [columns] string
    [extra] string
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Index");
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics
        .iter()
        .any(|d| matches!(&d.0.kind, DiagnosticKind::DuplicateRestAttribute))
    );
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::InvalidFieldTyp { field_name } if field_name == "columns"
    )));
  }

  #[test]
  fn reference_field_on_type() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    name declaration[Table] [label, unique]
  }

  type Ref [field] {
    target reference[Table]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Ref");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let lazy = element_typ.lookup_field_typ(&db, "target");
    assert!(lazy.is_some(), "expected field 'target' to exist");
    let resolved = lazy.unwrap().resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::Reference(_))),
      "expected Reference type for reference field, got (non-Debug type)"
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn qualified_reference_field_on_type() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    name qualified_declaration[Table] [label, unique]
  }

  type Ref [field] {
    target qualified_reference[Table]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Ref");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let lazy = element_typ.lookup_field_typ(&db, "target");
    assert!(lazy.is_some(), "expected field 'target' to exist");
    let resolved = lazy.unwrap().resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::QualifiedReference(_))),
      "expected QualifiedReference type, got (non-Debug type)"
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }
}

mod labels {
  use super::*;

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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::InvalidDeclarationFieldLabel { field_name, .. } if field_name == "name"
    )));
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty());
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::ConflictingLabelAnnotations { field_name } if field_name == "name"
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::UniqueRequiresLabel { field_name } if field_name == "name"
    )));
  }

  #[test]
  fn label_on_non_declaration_field_has_no_namespace_exposure() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Tag [element] {
    name string [label]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Tag");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let labels = element_typ.typ_element_labels(&db);
    assert_eq!(labels.len(), 1);
    let label = labels.iter().find(|l| l.field_name == "name").unwrap();
    assert!(!label.is_alias);
    assert!(!label.exposes_namespace);

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn alias_label_without_primary_label_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    alias qualified_declaration[Table] [label alias]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Table");
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::AliasLabelWithoutPrimaryLabel { field_name } if field_name == "alias"
    )));
  }

  #[test]
  fn alias_label_with_primary_label_emits_no_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    name declaration[Table] [label]
    alias qualified_declaration[Table] [label alias]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Table");
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      !diagnostics.iter().any(|d| matches!(
        &d.0.kind,
        DiagnosticKind::AliasLabelWithoutPrimaryLabel { .. }
      )),
      "unexpected AliasLabelWithoutPrimaryLabel diagnostic"
    );
  }

  #[test]
  fn qualified_declaration_label_exposes_namespace() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    name declaration[Table] [label]
    alias qualified_declaration[Table] [label alias]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Table");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let labels = element_typ.typ_element_labels(&db);
    let name_label = labels.iter().find(|l| l.field_name == "name").unwrap();
    assert!(
      !name_label.exposes_namespace,
      "declaration label must not expose namespace"
    );

    let alias_label = labels.iter().find(|l| l.field_name == "alias").unwrap();
    assert!(
      alias_label.exposes_namespace,
      "qualified_declaration alias label must expose namespace"
    );
  }

  #[test]
  fn two_primary_labels_both_collected() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Edge [element] {
    from declaration[Edge] [label]
    to declaration[Edge] [label]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Edge");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let labels = element_typ.typ_element_labels(&db);
    assert_eq!(labels.len(), 2, "expected two label definitions");
    assert!(labels.iter().any(|l| l.field_name == "from"));
    assert!(labels.iter().any(|l| l.field_name == "to"));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }
}

mod args {
  use super::*;

  #[test]
  fn arg_fields_exact_index_and_not_variadic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Ref [field] {
    from string [arg: 0]
    to string [arg: 1]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Ref");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let arg_fields = element_typ.typ_element_arg_fields(&db);
    assert_eq!(arg_fields.len(), 2);

    let arg0 = arg_fields.get(&0).expect("expected arg at index 0");
    assert_eq!(arg0.name, "from");
    assert!(!arg0.is_variadic);
    assert!(!arg0.has_csv);

    let arg1 = arg_fields.get(&1).expect("expected arg at index 1");
    assert_eq!(arg1.name, "to");
    assert!(!arg1.is_variadic);
    assert!(!arg1.has_csv);

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn arg_fields_variadic_flag() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Group [element] {
    name string [arg: 0]
    items string [arg: 1..]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Group");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let arg_fields = element_typ.typ_element_arg_fields(&db);
    let arg0 = arg_fields.get(&0).expect("expected arg at index 0");
    assert!(!arg0.is_variadic);

    let arg1 = arg_fields.get(&1).expect("expected arg at index 1");
    assert!(
      arg1.is_variadic,
      "variadic arg must have is_variadic = true"
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn arg_fields_csv_flag() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Index [element] {
    columns string [arg: 0, csv]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Index");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let arg_fields = element_typ.typ_element_arg_fields(&db);
    let arg0 = arg_fields.get(&0).expect("expected arg at index 0");
    assert_eq!(arg0.name, "columns");
    assert!(arg0.has_csv, "field with [csv] must have has_csv = true");
    assert!(!arg0.is_variadic);

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::CsvRequiresArg { field_name } if field_name == "values"
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::NonContiguousFieldArgIndex {
        field_name,
        found_index: 2,
        expected_index: 1,
      } if field_name == "to"
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::ArgAfterVariadicArg { field_name } if field_name == "extra"
    )));
  }
}

mod nested_typs {
  use super::*;

  #[test]
  fn element_instance_in_type_body_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    name declaration[Table] [label]
  }

  type Outer [element] {
    Table foo {}
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Outer");
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::DisallowedNestedElement { element_typ } if element_typ == "Table"
    )));
  }

  #[test]
  fn multiple_field_nested_typs_all_collected() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    type Column [field] {
      col_name string
    }
    type Index [field] {
      idx_name string
    }
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Table");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element_typ
        .typ_nested_typs(&db)
        .field_nested_typs
        .contains_key("Column")
    );
    assert!(
      element_typ
        .typ_nested_typs(&db)
        .field_nested_typs
        .contains_key("Index")
    );
    assert!(
      !element_typ
        .typ_nested_typs(&db)
        .element_nested_typs
        .contains_key("Column")
    );
    assert!(
      !element_typ
        .typ_nested_typs(&db)
        .element_nested_typs
        .contains_key("Index")
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn multiple_element_nested_typs_all_collected() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type TableGroup [element] {
    type Table [element] {
      name string
    }
    type View [element] {
      query string
    }
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "TableGroup");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element_typ
        .typ_nested_typs(&db)
        .element_nested_typs
        .contains_key("Table")
    );
    assert!(
      element_typ
        .typ_nested_typs(&db)
        .element_nested_typs
        .contains_key("View")
    );
    assert!(
      !element_typ
        .typ_nested_typs(&db)
        .field_nested_typs
        .contains_key("Table")
    );
    assert!(
      !element_typ
        .typ_nested_typs(&db)
        .field_nested_typs
        .contains_key("View")
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn nested_type_own_fields_fully_evaluated() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    type Column [field] {
      col_name string
      nullable bool? [setting]
    }
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Table");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let column_lazy = element_typ
      .typ_nested_typs(&db)
      .field_nested_typs
      .get("Column")
      .cloned();
    assert!(
      column_lazy.is_some(),
      "expected Column in nested_typs.field"
    );
    let column_typ = column_lazy.unwrap().resolve(&db);
    assert!(
      matches!(column_typ, Some(Typ::Element(_))),
      "expected Column to be an Element type"
    );
    let Typ::Element(column_elem) = column_typ.unwrap() else {
      unreachable!()
    };

    assert!(column_elem.typ_field_lines(&db).contains_key("col_name"));
    assert!(
      column_elem
        .typ_element_settings(&db)
        .contains_key("nullable")
    );
  }

  #[test]
  fn nested_types_same_name_different_parent_are_not_compatible() {
    // Types are nominal: two nested types with the same name in different parent types
    // are distinct types, even if structurally identical
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    name declaration[Table] [label]
    type Column [field] {
      col_name string [arg: 0]
    }
  }

  type Index [element] {
    name declaration[Index] [label]
    type Column [field] {
      col_name string [arg: 0]
    }
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let table_sym = get_first_symbol(&members, "Table");
    let index_sym = get_first_symbol(&members, "Index");

    let Some(Obj::Typ(Typ::Element(table_typ))) = evaluate_typ_symbol(&db, table_sym) else {
      panic!("expected Table to be ElementTyp");
    };
    let Some(Obj::Typ(Typ::Element(index_typ))) = evaluate_typ_symbol(&db, index_sym) else {
      panic!("expected Index to be ElementTyp");
    };

    // Both have a nested Column type
    let table_column = table_typ
      .typ_nested_typs(&db)
      .field_nested_typs
      .get("Column")
      .and_then(|t| t.resolve(&db));
    let index_column = index_typ
      .typ_nested_typs(&db)
      .field_nested_typs
      .get("Column")
      .and_then(|t| t.resolve(&db));

    assert!(table_column.is_some(), "Table should have nested Column");
    assert!(index_column.is_some(), "Index should have nested Column");
    // Nominal: they are different types even though they have the same name and structure
    assert!(
      table_column.unwrap() != index_column.unwrap(),
      "Table::Column and Index::Column must be different types (nominal)"
    );
  }

  #[test]
  fn nested_type_without_role_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    type Column {
      name string
    }
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Table");
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics
        .iter()
        .any(|d| matches!(&d.0.kind, DiagnosticKind::NestedTypMissingRole { typ_name } if typ_name == "Column")),
      "expected NestedTypMissingRole diagnostic for Column"
    );
  }

  #[test]
  fn nested_type_without_role_is_not_added_to_nested_typs() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    type Column {
      name string
    }
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Table");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected ElementTyp");
    };
    let nested = element_typ.typ_nested_typs(&db);
    assert!(
      !nested.field_nested_typs.contains_key("Column"),
      "Column without role should not be in field_nested_typs"
    );
    assert!(
      !nested.element_nested_typs.contains_key("Column"),
      "Column without role should not be in element_nested_typs"
    );
  }
}

mod namespace {
  use super::*;

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
    let Some(Obj::Typ(Typ::Element(element))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element.typ_namespace(&db).contains_key("name"));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
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
    let Some(Obj::Typ(Typ::Element(element))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element.typ_namespace(&db).contains_key("display_name"));
    assert!(element.vtable(&db).contains_key("display_name"));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn namespace_getter_no_return_type_defaults_to_any() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Column [field] {
    namespace {
      get computed() {}
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element_typ.typ_namespace(&db).contains_key("computed"));
    let resolved = element_typ.lookup_namespace_member_typ(&db, "computed");
    assert!(
      matches!(resolved, Some(Typ::Any(_))),
      "expected Any type for getter without return type annotation, got (non-Debug type)"
    );

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn namespace_multiple_field_reexports_all_visible() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Column [field] {
    name string
    type_info string

    namespace {
      name
      type_info
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element_typ.typ_namespace(&db).contains_key("name"));
    assert!(element_typ.typ_namespace(&db).contains_key("type_info"));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let resolved = element.lookup_namespace_member_typ(&db, "name");
    assert!(resolved.is_some());
    assert!(matches!(resolved.unwrap(), Typ::String(_)));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::UnknownNamespaceExport { field_name } if field_name == "nonexistent"
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::ColonDeclarationInNamespaceBlock { field_name } if field_name == "name"
    )));
  }
}

mod member_functions {
  use super::*;

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
    let Some(Obj::Typ(Typ::Element(element))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element.vtable(&db).contains_key("is_nullable"));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn member_function_with_typed_params_appears_in_vtable() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Column [field] {
    fn format(prefix: string, suffix: string): string {}
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Column");
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element_typ.vtable(&db).contains_key("format"));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }
}

mod constraints {
  use super::*;

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
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let constraint_defs = element_typ.constraint_definitions(&db);
    assert_eq!(constraint_defs.len(), 1);
    assert_eq!(constraint_defs[0].name, "nullability");

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty());
  }

  #[test]
  fn multiple_constraints_all_collected() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Column [field] {
    null bool? [setting]
    not_null bool? [setting]

    constraints {
      nullability: () => null ^ not_null
      always_valid: () => true
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let constraints = element_typ.constraint_definitions(&db);
    assert_eq!(constraints.len(), 2, "expected two constraint definitions");
    assert!(constraints.iter().any(|c| c.name == "nullability"));
    assert!(constraints.iter().any(|c| c.name == "always_valid"));

    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
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
    let diagnostics = evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::InvalidConstraintDefinition { constraint_name, .. } if constraint_name == "invalid_rule"
    )));
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
    let Some(Obj::Typ(Typ::Element(element))) = evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let constraint = element.lookup_constraint(&db, "no_nulls");
    assert!(constraint.is_some());
    assert_eq!(constraint.unwrap().name, "no_nulls");
    assert!(element.lookup_constraint(&db, "nonexistent").is_none());
  }
}
