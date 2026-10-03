use std::collections::HashMap;

use itertools::Either;

use crate::db::DboxideDatabase;
use crate::db::pretype_analyze::pretype_get_scope_members;
use crate::db::pretype_evaluate_typ::pretype_evaluate_typ_symbol;
use crate::db::test_utils::*;
use crate::db::types::interned::symbol::{StaticScopeMembers, VirtualModuleKind};
use crate::db::types::{Diagnostics, File, Obj, StaticScope, StaticScopeKind, StaticTyp};
use crate::diagnostics::DiagnosticKind;
use crate::{OnUsePolicy, Typ};

fn get_file_scope_members<'db>(db: &'db DboxideDatabase, file: File) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
  pretype_get_scope_members(db, scope).clone()
}

mod roles {
  use super::*;

  #[test]
  fn unrecognized_typ_role_emits_diagnostic() {
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
  type Baz [element, use: expand namespaces] {}
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    for name in ["Foo", "Bar", "Baz"] {
      let symbol = get_first_symbol(&members, name);
      let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
  fn empty_typ_body_has_no_fields() {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element_typ.typ_field_lines(&db).is_empty());
    assert!(element_typ.typ_element_labels(&db).is_empty());
    assert!(element_typ.typ_element_settings(&db).is_empty());
    assert!(element_typ.typ_element_attributes(&db).named.is_empty());

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

  type TableGroup [element, use: expand namespaces] {
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

    assert_eq!(
      element.typ_settings(&db).on_use,
      OnUsePolicy::ExpandReferences
    );
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.0.kind, DiagnosticKind::ConflictingTypRoles)),
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

  type TableGroup [element, use: inline] {
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
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::UnrecognizedOnUseValue { value } if value == "inline"
    )));
  }
}

mod fields {
  use super::*;

  #[test]
  fn pretype_evaluate_typ_element_symbol() {
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

    let labels = element.typ_element_labels(&db);
    assert_eq!(labels.len(), 2);
    let name_label = labels
      .iter()
      .find(|label| label.field_name == "name")
      .unwrap();
    assert!(!name_label.is_alias);
    assert!(name_label.unique);

    let alias_label = labels
      .iter()
      .find(|label| label.field_name == "alias")
      .unwrap();
    assert!(alias_label.is_alias);
    assert!(!alias_label.unique);

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
    assert!(namespace.members.contains_key("name"));
    assert!(namespace.members.contains_key("all_columns"));
    assert!(namespace.getters.contains_key("all_columns"));

    let vtable = element.vtable(&db);

    assert!(vtable.contains_key("count"));

    assert!(element.lookup_field_typ(&db, "name").is_some());

    assert!(element.lookup_field_typ(&db, "count").is_some());

    assert!(element.get_namespace_member(&db, "name").is_some());
    assert!(element.get_namespace_member(&db, "all_columns").is_some());
    assert!(element.get_namespace_member(&db, "nonexistent").is_none());

    let field_definitions = element.field_definitions(&db);
    assert!(field_definitions.iter().any(|field| field.name == "name"));
    assert!(
      field_definitions
        .iter()
        .any(|field| field.name == "headercolor")
    );

    let attribute_definitions = element.attribute_definitions(&db);
    assert!(
      attribute_definitions
        .iter()
        .any(|attribute| attribute.name == "headercolor")
    );

    let setting_definitions = element.setting_definitions(&db);
    assert!(
      setting_definitions
        .iter()
        .any(|setting| setting.name == "pk")
    );

    let field_nested_typ_definitions = element.field_nested_typ_definitions(&db);
    assert!(
      field_nested_typ_definitions
        .iter()
        .any(|definition| definition.name == "Column")
    );
    assert!(
      !field_nested_typ_definitions
        .iter()
        .any(|definition| definition.name == "Index")
    );

    let element_nested_typ_definitions = element.element_nested_typ_definitions(&db);
    assert!(
      element_nested_typ_definitions
        .iter()
        .any(|definition| definition.name == "Index")
    );
    assert!(
      !element_nested_typ_definitions
        .iter()
        .any(|definition| definition.name == "Column")
    );

    let nested_element_definitions = element.nested_typ_definitions(&db);
    assert!(
      nested_element_definitions
        .iter()
        .any(|definition| definition.name == "Index")
    );
    assert!(
      !nested_element_definitions
        .iter()
        .any(|definition| definition.name == "Column")
    );
  }

  #[test]
  fn nullable_field_typ_resolves_to_sum() {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let lazy = element_typ.lookup_field_typ(&db, "name");
    assert!(lazy.is_some(), "expected field 'name' to exist");
    let resolved = lazy.unwrap().resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::Sum(_))),
      "expected Sum type for nullable field, got (non-Debug type)"
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn generic_parameterized_field_typ_resolves_to_list() {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let lazy = element_typ.lookup_field_typ(&db, "tags");
    assert!(lazy.is_some(), "expected field 'tags' to exist");
    let resolved = lazy.unwrap().resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::List(_))),
      "expected List type for generic field, got (non-Debug type)"
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn invalid_field_typ_emits_diagnostic() {
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
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::InvalidFieldTyp { field_name } if field_name == "name"
    )));
  }

  #[test]
  fn colon_syntax_in_typ_body_emits_diagnostic() {
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
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element_typ
        .typ_element_attributes(&db)
        .named
        .contains_key("note")
    );
    assert!(element_typ.typ_element_settings(&db).contains_key("note"));

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

    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element_typ
        .typ_element_settings(&db)
        .contains_key("reference")
    );
    assert!(element_typ.typ_element_settings(&db).contains_key("dep"));

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element_typ.typ_element_attributes(&db).rest.is_some(),
      "expected rest attribute to be set"
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
    assert!(
      diagnostics
        .iter()
        .any(|diagnostic| matches!(&diagnostic.0.kind, DiagnosticKind::MalformedRestAttribute))
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics
        .iter()
        .any(|diagnostic| matches!(&diagnostic.0.kind, DiagnosticKind::DuplicateRestAttribute))
    );
  }

  #[test]
  fn rest_field_invalid_typ_emits_diagnostic() {
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
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::InvalidFieldTyp { field_name } if field_name == "columns"
    )));
  }

  #[test]
  fn reference_field_on_typ() {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let lazy = element_typ.lookup_field_typ(&db, "target");
    assert!(lazy.is_some(), "expected field 'target' to exist");
    let resolved = lazy.unwrap().resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::Reference(_))),
      "expected Reference type for reference field, got (non-Debug type)"
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn qualified_reference_field_on_typ() {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let lazy = element_typ.lookup_field_typ(&db, "target");
    assert!(lazy.is_some(), "expected field 'target' to exist");
    let resolved = lazy.unwrap().resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::QualifiedReference(_))),
      "expected QualifiedReference type, got (non-Debug type)"
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn attribute_field_in_field_typ_non_bool_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Column [field] {
  name string [arg: 0, label, unique]
  pk string [attribute]
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
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::FieldAttributeMustBeBool { field_name } if field_name == "pk"
      )),
      "expected FieldAttributeMustBeBool for 'pk'"
    );
  }

  #[test]
  fn attribute_field_in_field_typ_bool_no_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Column [field] {
  name string [arg: 0, label, unique]
  pk bool? [attribute, setting]
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
    assert!(
      !diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::FieldAttributeMustBeBool { .. }
      )),
      "unexpected FieldAttributeMustBeBool diagnostic"
    );
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let labels = element_typ.typ_element_labels(&db);
    assert_eq!(labels.len(), 1);
    let label = labels
      .iter()
      .find(|label| label.field_name == "name")
      .unwrap();
    assert!(!label.is_alias);

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      !diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::AliasLabelWithoutPrimaryLabel { .. }
      )),
      "unexpected AliasLabelWithoutPrimaryLabel diagnostic"
    );
  }

  #[test]
  fn two_primary_labels_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateLabelField { .. }
      )),
      "expected DuplicateLabelField diagnostic"
    );
  }

  #[test]
  fn two_alias_labels_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    name declaration[Table] [label]
    alias1 declaration[Table] [label alias]
    alias2 declaration[Table] [label alias]
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
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateLabelAliasField { .. }
      )),
      "expected DuplicateLabelAliasField diagnostic"
    );
  }

  fn conflicting_annotations_diagnostic(source: &str, field_name: &str, first: &str, second: &str) {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      source,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );
    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "T");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::ConflictingFieldAnnotations { field_name: f, first: a, second: b }
          if f == field_name && a == first && b == second
      )),
      "expected ConflictingFieldAnnotations({field_name}, {first}, {second})"
    );
  }

  #[test]
  fn label_and_setting_emits_conflicting_annotations() {
    conflicting_annotations_diagnostic(
      r#"
  use * from 'metatype'
  type T [element] { name string [label, setting] }
  "#,
      "name",
      "label",
      "setting",
    );
  }

  #[test]
  fn label_and_attribute_emits_conflicting_annotations() {
    conflicting_annotations_diagnostic(
      r#"
  use * from 'metatype'
  type T [element] { name string [label, attribute] }
  "#,
      "name",
      "label",
      "attribute",
    );
  }

  #[test]
  fn label_and_arg_coexist_without_conflict() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'
  type T [element] { name string [label, arg: 0] }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );
    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "T");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      !diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::ConflictingFieldAnnotations { .. }
      )),
      "[label] and [arg] should not conflict"
    );
  }

  #[test]
  fn setting_and_arg_emits_conflicting_annotations() {
    conflicting_annotations_diagnostic(
      r#"
  use * from 'metatype'
  type T [element] { name string [setting, arg: 0] }
  "#,
      "name",
      "setting",
      "arg",
    );
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
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

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
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

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let arg_fields = element_typ.typ_element_arg_fields(&db);
    let arg0 = arg_fields.get(&0).expect("expected arg at index 0");
    assert_eq!(arg0.name, "columns");
    assert!(arg0.has_csv, "field with [csv] must have has_csv = true");
    assert!(!arg0.is_variadic);

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
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
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::ArgAfterVariadicArg { field_name } if field_name == "extra"
    )));
  }

  #[test]
  fn arg_field_text_flag_set() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Column [field] {
    name string [arg: 0]
    col_type string? [arg: 1, text]
  }
  "#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Column");
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let arg_fields = element_typ.typ_element_arg_fields(&db);
    let arg1 = arg_fields.get(&1).expect("expected arg at index 1");
    assert_eq!(arg1.name, "col_type");
    assert!(arg1.is_text, "expected is_text to be true for [text] field");

    let arg0 = arg_fields.get(&0).expect("expected arg at index 0");
    assert!(
      !arg0.is_text,
      "expected is_text to be false for non-[text] field"
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }
}

mod nested_typs {
  use super::*;

  #[test]
  fn element_instance_in_typ_body_emits_diagnostic() {
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::DisallowedNestedElement { element_typ } if element_typ == "Table"
    )));
  }

  #[test]
  fn multiple_field_nested_typs_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    // First [field] type is accepted, second emits a diagnostic and is not collected
    assert!(
      element_typ
        .typ_nested_typs(&db)
        .field_nested_typs
        .contains_key("Column")
    );
    assert!(
      !element_typ
        .typ_nested_typs(&db)
        .field_nested_typs
        .contains_key("Index")
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics
        .iter()
        .any(|diagnostic| matches!(&diagnostic.0.kind, DiagnosticKind::MultipleNestedFieldTyps))
    );
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
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

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn nested_typ_own_fields_fully_evaluated() {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
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
    let Typ::Element(column_element) = column_typ.unwrap() else {
      unreachable!()
    };

    assert!(column_element.typ_field_lines(&db).contains_key("col_name"));
    assert!(
      column_element
        .typ_element_settings(&db)
        .contains_key("nullable")
    );
  }

  #[test]
  fn nested_typs_same_name_different_parent_are_not_compatible() {
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
    let table_symbol = get_first_symbol(&members, "Table");
    let index_symbol = get_first_symbol(&members, "Index");

    let Some(Obj::Typ(Typ::Element(table_typ))) = pretype_evaluate_typ_symbol(&db, table_symbol)
    else {
      panic!("expected Table to be ElementTyp");
    };
    let Some(Obj::Typ(Typ::Element(index_typ))) = pretype_evaluate_typ_symbol(&db, index_symbol)
    else {
      panic!("expected Index to be ElementTyp");
    };

    // Both have a nested Column type
    let table_column = table_typ
      .typ_nested_typs(&db)
      .field_nested_typs
      .get("Column")
      .and_then(|typ| typ.resolve(&db));
    let index_column = index_typ
      .typ_nested_typs(&db)
      .field_nested_typs
      .get("Column")
      .and_then(|typ| typ.resolve(&db));

    assert!(table_column.is_some(), "Table should have nested Column");
    assert!(index_column.is_some(), "Index should have nested Column");
    // Nominal: they are different types even though they have the same name and structure
    assert!(
      table_column.unwrap() != index_column.unwrap(),
      "Table::Column and Index::Column must be different types (nominal)"
    );
  }

  #[test]
  fn nested_typ_without_role_emits_diagnostic() {
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics
        .iter()
        .any(|diagnostic| matches!(&diagnostic.0.kind, DiagnosticKind::NestedTypMissingRole { typ_name } if typ_name == "Column")),
      "expected NestedTypMissingRole diagnostic for Column"
    );
  }

  #[test]
  fn nested_typ_without_role_is_not_added_to_nested_typs() {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
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
    let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element.typ_namespace(&db).members.contains_key("name"));

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element
        .typ_namespace(&db)
        .members
        .contains_key("display_name")
    );
    assert!(
      element
        .typ_namespace(&db)
        .getters
        .contains_key("display_name")
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn namespace_getter_no_return_typ_defaults_to_any() {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(
      element_typ
        .typ_namespace(&db)
        .members
        .contains_key("computed")
    );
    let resolved = element_typ.lookup_namespace_member_typ(&db, "computed");
    assert!(
      matches!(resolved, Some(Typ::Obj(_))),
      "expected Any type for getter without return type annotation, got (non-Debug type)"
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element_typ.typ_namespace(&db).members.contains_key("name"));
    assert!(
      element_typ
        .typ_namespace(&db)
        .members
        .contains_key("type_info")
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn lookup_namespace_member_typ_resolves_eager_typ() {
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
    assert!(resolved.is_some());
    assert!(matches!(resolved.unwrap(), Typ::String(_)));

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::ColonDeclarationInNamespaceBlock { field_name } if field_name == "name"
    )));
  }

  #[test]
  fn namespace_expanded_field_sets_expanded_on_list_field() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Column [field] {
    name string
  }

  type Table [element] {
    columns Column[]

    namespace {
      columns [expand]
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
    let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let namespace = element.typ_namespace(&db);
    assert_eq!(namespace.expanded_field, Some("columns".to_string()));
    assert!(namespace.expanded_item_typ.is_some());

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }

  #[test]
  fn namespace_expanded_field_static_lookup_returns_item_typ() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
  use * from 'metatype'

  type Column [field] {
    name string
  }

  type Table [element] {
    columns Column[]

    namespace {
      columns [expand]
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
    let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    // Any name not in members/getters resolves to the item type (non-nullable) via expanded_item_typ
    let resolved = element.lookup_namespace_member_typ(&db, "id");
    assert!(
      matches!(resolved, Some(Typ::Element(_))),
      "expected Element type for expanded field static lookup"
    );
  }

  #[test]
  fn namespace_expanded_duplicate_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Column [field] {
    name string
  }

  type Table [element] {
    columns Column[]
    indexes Column[]

    namespace {
      columns [expand]
      indexes [expand]
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::DuplicateExpandedNamespaceField { .. }
      )),
      "expected DuplicateExpandedNamespaceField diagnostic"
    );
  }

  #[test]
  fn namespace_expanded_on_non_list_field_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    name string

    namespace {
      name [expand]
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::ExpandedNamespaceFieldMustBeList { .. }
      )),
      "expected ExpandedNamespaceFieldMustBeList diagnostic"
    );
  }

  #[test]
  fn namespace_expanded_unknown_field_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "invalid.dbml",
      r#"
  use * from 'metatype'

  type Table [element] {
    namespace {
      nonexistent [expand]
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.iter().any(|diagnostic| matches!(
        &diagnostic.0.kind,
        DiagnosticKind::UnknownNamespaceExport { field_name } if field_name == "nonexistent"
      )),
      "expected UnknownNamespaceExport diagnostic"
    );
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
    let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element.vtable(&db).contains_key("is_nullable"));

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    assert!(element_typ.vtable(&db).contains_key("format"));

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.is_empty(), "unexpected diagnostics");
  }
}

mod constraints {
  use super::*;

  #[test]
  fn posttype_evaluate_element_typ_with_constraints() {
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let constraint_definitions = element_typ.constraint_definitions(&db);
    assert_eq!(constraint_definitions.len(), 1);
    assert_eq!(constraint_definitions[0].name, "nullability");

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let constraints = element_typ.constraint_definitions(&db);
    assert_eq!(constraints.len(), 2, "expected two constraint definitions");
    assert!(
      constraints
        .iter()
        .any(|constraint| constraint.name == "nullability")
    );
    assert!(
      constraints
        .iter()
        .any(|constraint| constraint.name == "always_valid")
    );

    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
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
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
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
    let Some(Obj::Typ(Typ::Element(element))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };

    let constraint = element.lookup_constraint(&db, "no_nulls");
    assert!(constraint.is_some());
    assert_eq!(constraint.unwrap().name, "no_nulls");
    assert!(element.lookup_constraint(&db, "nonexistent").is_none());
  }
}

mod array_types {
  use super::*;

  #[test]
  fn array_annotation_on_field_resolves_to_list_typ() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
use * from 'metatype'

type Container [element] {
  items string[] [arg: 0..]
}
"#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Container");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.is_empty(),
      "string[] should resolve with no diagnostics"
    );

    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };
    let arg_fields = element_typ.typ_element_arg_fields(&db);
    let items_field = arg_fields.get(&0).expect("expected arg at index 0");
    assert_eq!(items_field.name, "items");
    // Type should be ListTyp[StringTyp]
    let resolved = items_field.typ.resolve(&db);
    assert!(matches!(resolved, Some(Typ::List(_))), "expected ListTyp");
  }

  #[test]
  fn nested_generic_array_resolves() {
    // reference[Table][] should produce ListTyp[ReferenceTyp[Table]]
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "valid.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name string [label]
}

type Group [element] {
  tables reference[Table][] [arg: 0..]
}
"#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Group");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.is_empty(),
      "reference[Table][] should resolve with no diagnostics"
    );

    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };
    let arg_fields = element_typ.typ_element_arg_fields(&db);
    let tables_field = arg_fields.get(&0).expect("expected arg at index 0");
    let resolved = tables_field.typ.resolve(&db);
    assert!(matches!(resolved, Some(Typ::List(_))), "expected ListTyp");
  }
}

mod virtual_functions {
  use super::*;

  fn metatype_file(db: &DboxideDatabase, source: &str) -> File {
    create_file(
      db,
      "test.dbml",
      source,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    )
  }

  #[test]
  fn interpret_of_known_type_resolves_to_that_type() {
    // interpret(string) = string at pretype time
    let db = DboxideDatabase::default();
    let file = metatype_file(
      &db,
      r#"
use * from 'metatype'

type Row [element] {
  value interpret(string)? [setting]
}
"#,
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Row");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.is_empty(),
      "interpret(string)? should resolve with no diagnostics"
    );

    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };
    let field = element_typ
      .field_definitions(&db)
      .into_iter()
      .find(|field| field.name == "value")
      .expect("expected field 'value'");
    // interpret(string)? -> string? -> SumTyp([StringTyp, NullTyp])
    let resolved = field.typ.resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::Sum(_))),
      "expected SumTyp (string?) from interpret(string)?"
    );
  }

  #[test]
  fn interpret_with_unresolvable_arg_falls_back_to_any() {
    // interpret(unknownRef)? -> any? when arg can't be resolved
    let db = DboxideDatabase::default();
    let file = metatype_file(
      &db,
      r#"
use * from 'metatype'

type Row [element] {
  value interpret(type)? [setting]
}
"#,
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Row");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.is_empty(),
      "interpret(type)? should resolve to any? with no diagnostics"
    );
  }

  #[test]
  fn typeof_of_string_literal_resolves_to_string_typ() {
    // typeof('hello') = string at pretype time
    let db = DboxideDatabase::default();
    let file = metatype_file(
      &db,
      r#"
use * from 'metatype'

type Row [element] {
  value typeof('hello') [setting]
}
"#,
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Row");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.is_empty(),
      "typeof('hello') should resolve to string with no diagnostics"
    );

    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };
    let field = element_typ
      .field_definitions(&db)
      .into_iter()
      .find(|field| field.name == "value")
      .expect("expected field 'value'");
    let resolved = field.typ.resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::String(_))),
      "expected StringTyp from typeof('hello')"
    );
  }

  #[test]
  fn typeof_of_integer_literal_resolves_to_integer_typ() {
    let db = DboxideDatabase::default();
    let file = metatype_file(
      &db,
      r#"
use * from 'metatype'

type Row [element] {
  value typeof(42) [setting]
}
"#,
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Row");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.is_empty(),
      "typeof(42) should resolve to integer with no diagnostics"
    );

    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };
    let field = element_typ
      .field_definitions(&db)
      .into_iter()
      .find(|field| field.name == "value")
      .expect("expected field 'value'");
    let resolved = field.typ.resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::Integer(_))),
      "expected IntegerTyp from typeof(42)"
    );
  }

  #[test]
  fn interpret_named_import_resolves_call() {
    // `use { fn interpret }` brings interpret into scope and the call works
    let db = DboxideDatabase::default();
    let file = metatype_file(
      &db,
      r#"
use {
  fn interpret
  type string
} from 'metatype'

type Row [element] {
  value interpret(string) [setting]
}
"#,
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Row");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.is_empty(),
      "named import of interpret should resolve call with no diagnostics"
    );
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };
    let field = element_typ
      .field_definitions(&db)
      .into_iter()
      .find(|field| field.name == "value")
      .expect("expected field 'value'");
    assert!(
      matches!(field.typ.resolve(&db), Some(Typ::String(_))),
      "expected StringTyp from interpret(string)"
    );
  }

  #[test]
  fn interpret_aliased_import_resolves_call() {
    // `use { fn interpret as i }` brings interpret as `i` and `i(...)` works
    let db = DboxideDatabase::default();
    let file = metatype_file(
      &db,
      r#"
use {
  fn interpret as i
  type string
} from 'metatype'

type Row [element] {
  value i(string) [setting]
}
"#,
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Row");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.is_empty(),
      "aliased import of interpret as i should resolve call with no diagnostics"
    );
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };
    let field = element_typ
      .field_definitions(&db)
      .into_iter()
      .find(|field| field.name == "value")
      .expect("expected field 'value'");
    assert!(
      matches!(field.typ.resolve(&db), Some(Typ::String(_))),
      "expected StringTyp from i(string)"
    );
  }

  #[test]
  fn interpret_without_import_does_not_resolve() {
    // Without importing interpret, the call should not evaluate
    let db = DboxideDatabase::default();
    let file = metatype_file(
      &db,
      r#"
type Row [element] {
  value interpret(string) [setting]
}
"#,
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Row");
    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };
    let field = element_typ
      .field_definitions(&db)
      .into_iter()
      .find(|field| field.name == "value")
      .expect("expected field 'value'");
    // Without the import, interpret(string) cannot be resolved - field type falls back to any
    assert!(
      matches!(field.typ.resolve(&db), Some(Typ::Obj(_)) | None),
      "expected unresolved type when interpret is not imported"
    );
  }
}

mod dq_string {
  use super::*;

  #[test]
  fn dq_string_type_name_resolves_as_referee() {
    // Double-quoted names can reference types with spaces in their names via referee resolution
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
type "My Type" [element] {}

type Container [element] {
  value "My Type" [setting]
}
"#,
      HashMap::new(),
    );

    let members = get_file_scope_members(&db, file);
    let symbol = get_first_symbol(&members, "Container");
    let diagnostics = pretype_evaluate_typ_symbol::accumulated::<Diagnostics>(&db, symbol);
    assert!(
      diagnostics.is_empty(),
      r#""My Type" as field type should resolve with no diagnostics"#
    );

    let Some(Obj::Typ(Typ::Element(element_typ))) = pretype_evaluate_typ_symbol(&db, symbol) else {
      panic!("expected Typ::Element");
    };
    let field = element_typ
      .field_definitions(&db)
      .into_iter()
      .find(|field| field.name == "value")
      .expect("expected field 'value'");
    let resolved = field.typ.resolve(&db);
    assert!(
      matches!(resolved, Some(Typ::Element(_))),
      r#"expected ElementTyp from "My Type" referee"#
    );
  }
}
