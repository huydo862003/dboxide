use salsa::Database;

use crate::ast::{AstNode, BlockElementDeclaration, ElementDeclaration, RedNode};
use crate::db::pretype::pretype_evaluate::pretype_evaluate_typ_symbol::evaluate_typ_symbol;
use crate::db::pretype::pretype_resolve_name::resolve_name;
use crate::db::types::{Diagnostics, File, Obj, Typ};
use crate::db::utils::{ElementBodyExtract, extract_element_body};
use crate::diagnostics::Diagnostic;

// check an element instance body against its resolved ElementTyp
pub fn check_element(db: &dyn Database, file: File, element_node: &RedNode) {
  let Some(element) = ElementDeclaration::cast(element_node.clone()) else {
    return;
  };
  let Some(typ_node) = element.typ() else {
    return;
  };

  let fragments = typ_node.collect_path_fragments();
  let Some(typ_name_fragment) = fragments.last() else {
    return;
  };
  if typ_name_fragment.eq_ignore_ascii_case("type") {
    return;
  }

  // resolve element type
  let Some(typ_symbol) = *resolve_name(db, file, element_node.clone(), typ_name_fragment.clone())
  else {
    let start = typ_node.syntax().offset();
    salsa::Accumulator::accumulate(
      Diagnostics(Diagnostic::UnresolvedElementType {
        name: typ_name_fragment.clone(),
        start_offset: start,
        end_offset: start + typ_node.syntax().text_len(),
      }),
      db,
    );
    return;
  };

  let Some(Obj::Typ(Typ::Element(element_typ))) = *evaluate_typ_symbol(db, typ_symbol) else {
    return;
  };

  // extract body and validate
  let Some(block) = BlockElementDeclaration::cast(element_node.clone()) else {
    return;
  };
  let Some(body_node) = block.body() else {
    return;
  };
  let body = extract_element_body(&body_node);
  let typ_name = element_typ.name(db).clone();

  check_attributes(db, &body, &typ_name, element_typ.attributes(db));
  check_sub_elements(db, file, &body, &typ_name, element_typ.element_subtypes(db));
}

// validate key-value attributes against the type's declared attributes
fn check_attributes(
  db: &dyn Database,
  body: &ElementBodyExtract,
  typ_name: &str,
  declared_attributes: &std::collections::BTreeMap<
    String,
    crate::db::types::interned::lazy_typ::LazyTyp<'_>,
  >,
) {
  for attr in &body.attributes {
    let Some(attr_name) = attr.name().and_then(|n| n.name()).and_then(|e| e.as_name()) else {
      continue;
    };
    if !declared_attributes.contains_key(&attr_name) {
      let start = attr.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(Diagnostic::UnknownAttribute {
          name: attr_name,
          typ_name: typ_name.to_string(),
          start_offset: start,
          end_offset: start + attr.syntax().text_len(),
        }),
        db,
      );
    }
  }
}

// validate nested sub-element blocks against the type's declared element subtypes
fn check_sub_elements(
  db: &dyn Database,
  file: File,
  body: &ElementBodyExtract,
  typ_name: &str,
  declared_subtypes: &std::collections::BTreeMap<
    String,
    crate::db::types::interned::lazy_typ::LazyTyp<'_>,
  >,
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
        Diagnostics(Diagnostic::UnknownSubElement {
          name: nested_name,
          typ_name: typ_name.to_string(),
          start_offset: start,
          end_offset: start + nested.syntax().text_len(),
        }),
        db,
      );
    }

    // recurse into nested elements
    check_element(db, file, nested.syntax());
  }
}
