use std::collections::BTreeMap;

use salsa::Database;

use crate::ast::{AstNode, BlockElementDeclaration, ElementDeclaration, RedNode};
use crate::db::prepost::evaluate_node::evaluate_node_posttype;
use crate::db::prepost::{ElementBodyExtract, RuntimeScope, extract_element_body};
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{AttributeSchema, Diagnostics, File, Obj, Typ};

use super::validate_constraints::validate_constraints;
use crate::diagnostics::DiagnosticKind;

use crate::db::posttype::resolve::resolve_element_typ::resolve_element_typ;

pub fn validate_element_fields(db: &dyn Database, file: File, element_node: &RedNode) {
  let Some(element) = ElementDeclaration::cast(element_node.clone()) else {
    return;
  };
  let Some(typ_node) = element.typ() else {
    return;
  };

  let Some(element_typ) = resolve_element_typ(db, file, element_node) else {
    // Emit diagnostic only for non-type declarations with unresolvable types
    let fragments = typ_node.collect_path_fragments();
    if let Some(name) = fragments.last()
      && !name.eq_ignore_ascii_case("type")
    {
      let start = typ_node.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::UnresolvedElementTyp { name: name.clone() }
            .into_error(start, start + typ_node.syntax().text_len()),
        ),
        db,
      );
    }
    return;
  };

  let Some(block) = BlockElementDeclaration::cast(element_node.clone()) else {
    return;
  };
  let Some(body_node) = block.body() else {
    return;
  };
  let body = extract_element_body(&body_node);
  let typ_name = element_typ.name(db).clone();

  check_element_settings(
    db,
    &element,
    &typ_name,
    element_typ.typ_element_settings(db),
  );
  check_field_lines(
    db,
    &body,
    &element_typ.typ_nested_typs(db).field_nested_typs,
  );
  check_attributes(db, &body, &typ_name, element_typ.typ_element_attributes(db));
  check_sub_elements(
    db,
    file,
    &body,
    &typ_name,
    &element_typ.typ_nested_typs(db).element_nested_typs,
  );

  let field_values = collect_field_values(db, file, &element, &body);
  validate_constraints(db, file, element_typ, &field_values);
}

/// Build a map of field name -> evaluated Obj for constraint evaluation
/// Includes label fields (element name), attribute values, and setting values
fn collect_field_values<'db>(
  db: &'db dyn Database,
  file: File,
  element: &ElementDeclaration,
  body: &ElementBodyExtract,
) -> BTreeMap<String, Obj<'db>> {
  let mut values = BTreeMap::new();
  let scope = RuntimeScope::empty(db);

  // Label field: the element's declaration name (e.g. `users` in `Table users`)
  if let Some(name) = element.declaration_name() {
    use crate::db::types::StringObj;
    values.insert(name.clone(), StringObj::new(db, name).into());
  }

  // Attribute values: `key: expr` in element body
  for attribute in &body.attributes {
    let Some(name) = attribute
      .name()
      .and_then(|n| n.name())
      .and_then(|e| e.as_name())
    else {
      continue;
    };
    if let Some(expr) = attribute.value().and_then(|v| v.expr())
      && let Some(obj) = evaluate_node_posttype(db, file, &expr, scope)
    {
      values.insert(name, obj);
    }
  }

  // Setting values: `[key: val]` in element header
  if let Some(setting_list) = element.setting_list() {
    for item in setting_list.items() {
      let Some(name_node) = item.name() else {
        continue;
      };
      let name = name_node.name();
      if let Some(expr) = item.value().and_then(|v| v.expr())
        && let Some(obj) = evaluate_node_posttype(db, file, &expr, scope)
      {
        values.insert(name, obj);
      }
    }
  }

  values
}

fn check_attributes<'db>(
  db: &'db dyn Database,
  body: &ElementBodyExtract,
  typ_name: &str,
  declared_attributes: &AttributeSchema<'db>,
) {
  // If a rest attribute is declared, any key-value attribute is valid
  if declared_attributes.rest.is_some() {
    return;
  }
  for attribute in &body.attributes {
    let Some(attribute_name) = attribute
      .name()
      .and_then(|n| n.name())
      .and_then(|e| e.as_name())
    else {
      continue;
    };
    if !declared_attributes.named.contains_key(&attribute_name) {
      let start = attribute.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::UnknownAttribute {
            name: attribute_name,
            typ_name: typ_name.to_string(),
          }
          .into_error(start, start + attribute.syntax().text_len()),
        ),
        db,
      );
    }
  }
}

fn check_sub_elements<'db>(
  db: &'db dyn Database,
  file: File,
  body: &ElementBodyExtract,
  typ_name: &str,
  declared_subtypes: &BTreeMap<String, LazyTyp<'db>>,
) {
  for nested in &body.element_declarations {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    let Some(nested_typ) = element.typ() else {
      continue;
    };
    let nested_name = nested_typ.text().trim().to_string();
    let matches_subtype = declared_subtypes
      .keys()
      .any(|k| k.eq_ignore_ascii_case(&nested_name));

    if !matches_subtype {
      let start = nested.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::UnknownSubElement {
            name: nested_name,
            typ_name: typ_name.to_string(),
          }
          .into_error(start, start + nested.syntax().text_len()),
        ),
        db,
      );
    }

    validate_element_fields(db, file, nested.syntax());
  }
}

fn check_element_settings<'db>(
  db: &'db dyn Database,
  element: &ElementDeclaration,
  typ_name: &str,
  declared_settings: &BTreeMap<String, LazyTyp<'db>>,
) {
  let Some(setting_list) = element.setting_list() else {
    return;
  };
  for item in setting_list.items() {
    let Some(name_node) = item.name() else {
      continue;
    };
    let setting_name = name_node.name();
    if !declared_settings.contains_key(&setting_name) {
      let start = item.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::UnknownSetting {
            name: setting_name,
            typ_name: typ_name.to_string(),
          }
          .into_error(start, start + item.syntax().text_len()),
        ),
        db,
      );
    }
  }
}

fn check_field_lines<'db>(
  db: &'db dyn Database,
  body: &ElementBodyExtract,
  declared_field_nested_typs: &BTreeMap<String, LazyTyp<'db>>,
) {
  if declared_field_nested_typs.is_empty() {
    return;
  }
  for field_line in &body.field_declarations {
    let mut args = field_line.args();
    let Some(first_arg) = args.next() else {
      continue;
    };
    let Some(type_name) = first_arg.expr().and_then(|e| e.as_name()) else {
      continue;
    };
    let Some(lazy_typ) = declared_field_nested_typs
      .iter()
      .find(|(k, _)| k.eq_ignore_ascii_case(&type_name))
      .map(|(_, v)| v)
    else {
      continue;
    };
    let Some(Typ::Element(field_typ)) = lazy_typ.resolve(db) else {
      continue;
    };
    let arg_fields = field_typ.typ_element_arg_fields(db);
    if arg_fields.is_empty() {
      continue;
    }
    let actual_count = args.count();
    let has_variadic = arg_fields.values().any(|f| f.is_variadic);
    let non_variadic_count = arg_fields.values().filter(|f| !f.is_variadic).count();

    let start = field_line.syntax().offset();
    let end = start + field_line.syntax().text_len();

    if actual_count < non_variadic_count {
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::TooFewFieldLineArgs {
            typ_name: type_name.clone(),
            min_expected: non_variadic_count,
            found: actual_count,
          }
          .into_error(start, end),
        ),
        db,
      );
    } else if !has_variadic && actual_count > non_variadic_count {
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::TooManyFieldLineArgs {
            typ_name: type_name.clone(),
            max_expected: non_variadic_count,
            found: actual_count,
          }
          .into_error(start, end),
        ),
        db,
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use itertools::Either;

  use crate::db::DboxideDatabase;
  use crate::db::posttype::validate::validate_file_elements::validate_file_elements;
  use crate::db::test_utils::*;
  use crate::db::types::Diagnostics;
  use crate::db::types::interned::symbol::VirtualModuleKind;
  use crate::diagnostics::DiagnosticKind;

  fn metatype() -> HashMap<String, Either<VirtualModuleKind, crate::db::types::File>> {
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )])
  }

  #[test]
  fn unknown_attribute_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Project [element] {
  name declaration[Project] [label, unique]
}

Project myproject {
  nonexistent: 'value'
}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::UnknownAttribute { name, typ_name, .. }
        if name == "nonexistent" && typ_name == "Project"
    )));
  }

  #[test]
  fn unresolved_element_type_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

Nonexistent foo {}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::UnresolvedElementTyp { name, .. } if name == "Nonexistent"
    )));
  }

  #[test]
  fn unknown_sub_element_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
}

Table users {
  unknown_block {
    something
  }
}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::UnknownSubElement { name, typ_name, .. }
        if name == "unknown_block" && typ_name == "Table"
    )));
  }

  #[test]
  fn too_few_field_line_args_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  type Column [field] {
    col_name string [arg: 0]
    col_type string [arg: 1]
  }
}

Table users {
  Column id
}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::TooFewFieldLineArgs { typ_name, min_expected: 2, found: 1 }
        if typ_name == "Column"
    )));
  }

  #[test]
  fn too_many_field_line_args_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  type Column [field] {
    col_name string [arg: 0]
  }
}

Table users {
  Column id extra_arg
}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(diagnostics.iter().any(|d| matches!(
      &d.0.kind,
      DiagnosticKind::TooManyFieldLineArgs { typ_name, max_expected: 1, found: 2 }
        if typ_name == "Column"
    )));
  }

  #[test]
  fn correct_field_line_arg_count_emits_no_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
  type Column [field] {
    col_name string [arg: 0]
    col_type string [arg: 1]
  }
}

Table users {
  Column id integer
}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(
      !diagnostics.iter().any(|d| matches!(
        &d.0.kind,
        DiagnosticKind::TooFewFieldLineArgs { .. } | DiagnosticKind::TooManyFieldLineArgs { .. }
      )),
      "unexpected arg count diagnostic"
    );
  }

  #[test]
  fn variadic_arg_accepts_zero_or_more_values() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Index [element] {
  name declaration[Index] [label, unique]
  type IndexField [field] {
    col_name string [arg: 0]
    extra string [arg: 1..]
  }
}

Index pk_idx {
  IndexField id
  IndexField id email name
}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(
      !diagnostics.iter().any(|d| matches!(
        &d.0.kind,
        DiagnosticKind::TooFewFieldLineArgs { .. } | DiagnosticKind::TooManyFieldLineArgs { .. }
      )),
      "variadic arg should accept any count >= non_variadic_count"
    );
  }

  #[test]
  fn unknown_element_setting_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name declaration[Table] [label, unique]
}

Table users [badkey: red] {}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
      &diagnostic.0.kind,
      DiagnosticKind::UnknownSetting { name, .. } if name == "badkey"
    )));
  }

  #[test]
  fn rest_attribute_non_map_type_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Project [element] {
  name declaration[Project] [label, unique]
  [extra] string
}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(
      diagnostics
        .iter()
        .any(|d| matches!(&d.0.kind, DiagnosticKind::RestAttributeMustBeMap))
    );
  }

  #[test]
  fn rest_attribute_map_type_no_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Project [element] {
  name declaration[Project] [label, unique]
  [extra] Map
}
"#,
      metatype(),
    );

    let diagnostics = validate_file_elements::accumulated::<Diagnostics>(&db, file);
    assert!(
      !diagnostics
        .iter()
        .any(|d| matches!(&d.0.kind, DiagnosticKind::RestAttributeMustBeMap)),
      "unexpected RestAttributeMustBeMap diagnostic"
    );
  }
}
