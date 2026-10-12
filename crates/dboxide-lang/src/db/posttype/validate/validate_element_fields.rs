use std::collections::BTreeMap;

use salsa::Database;

use crate::ast::{AstNode, BlockElementDeclaration, ElementDeclaration, RedNode};
use crate::db::prepost::{ElementBodyExtract, extract_element_body};
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{Diagnostics, File};
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
          DiagnosticKind::UnresolvedElementType { name: name.clone() }
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

  check_element_settings(db, &element, &typ_name, element_typ.settings(db));
  check_field_lines(db, &body, &typ_name, element_typ.field_subtypes(db));
  check_attributes(db, &body, &typ_name, element_typ.attributes(db));
  check_sub_elements(db, file, &body, &typ_name, element_typ.element_subtypes(db));
}

fn check_attributes<'db>(
  db: &'db dyn Database,
  body: &ElementBodyExtract,
  typ_name: &str,
  declared_attributes: &BTreeMap<String, LazyTyp<'db>>,
) {
  for attr in &body.attributes {
    let Some(attr_name) = attr.name().and_then(|n| n.name()).and_then(|e| e.as_name()) else {
      continue;
    };
    if !declared_attributes.contains_key(&attr_name) {
      let start = attr.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::UnknownAttribute {
            name: attr_name,
            typ_name: typ_name.to_string(),
          }
          .into_error(start, start + attr.syntax().text_len()),
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
  _db: &'db dyn Database,
  body: &ElementBodyExtract,
  _typ_name: &str,
  declared_field_subtypes: &BTreeMap<String, LazyTyp<'db>>,
) {
  if declared_field_subtypes.len() != 1 {
    return;
  }
  for field in &body.field_declarations {
    let Some(_setting_list) = field.setting_list() else {
      continue;
    };
    // Field line settings validation is deferred to a future pass
  }
}
