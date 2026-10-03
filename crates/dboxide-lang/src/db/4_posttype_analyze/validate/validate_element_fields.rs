use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use salsa::{Accumulator, Database};

use crate::ast::BlockElementDeclarationBody;
use crate::ast::{AstNode, BlockElementDeclaration, CsvExpr, ElementDeclaration, RedNode};
use crate::db::get_declaration_typ;
use crate::db::posttype_analyze::declare_field_line_symbols;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{AttributeSchema, Diagnostics, File, Typ};
use crate::diagnostics::DiagnosticKind;

/// Validates field structure of a single element instance (settings, attributes, sub-elements, field lines)
pub fn validate_element_fields(db: &dyn Database, file: File, element_node: &RedNode) {
  let Some(element) = ElementDeclaration::cast(element_node.clone()) else {
    return;
  };

  let declaration_typ = *get_declaration_typ(db, file, element_node.clone());

  let Some(Typ::Element(element_typ)) = declaration_typ else {
    let Some(typ_node) = element.typ() else {
      return;
    };

    if let Some(Typ::TypTyp(_)) = declaration_typ {
      return;
    }

    let Some(name_fragment) = typ_node.name() else {
      return;
    };
    let name = name_fragment.name().to_string();

    Diagnostics::accumulate(
      Diagnostics(DiagnosticKind::UnresolvedElementTyp { name }.into_error(
        typ_node.syntax().start_offset(),
        typ_node.syntax().end_offset(),
      )),
      db,
    );
    return;
  };

  let Some(block) = BlockElementDeclaration::cast(element_node.clone()) else {
    return;
  };
  let Some(body) = block.body() else {
    return;
  };
  let typ_name = element_typ.name(db).clone();

  check_element_settings(
    db,
    &element,
    &typ_name,
    element_typ.typ_element_settings(db),
    element_typ.typ_unique_settings(db),
  );
  check_field_lines(
    db,
    file,
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
}

/// Validates that all attributes on the element are declared by its type and not duplicated
fn check_attributes<'db>(
  db: &'db dyn Database,
  body: &BlockElementDeclarationBody,
  typ_name: &str,
  declared_attributes: &AttributeSchema<'db>,
) {
  // If a rest attribute is declared, any key-value attribute is valid
  if declared_attributes.rest.is_some() {
    return;
  }
  let mut seen: HashSet<String> = HashSet::new();
  for attribute in body.attributes() {
    let Some(attribute_name) = attribute
      .name()
      .and_then(|name_node| name_node.name())
      .and_then(|expr| expr.as_name())
    else {
      continue;
    };
    if !declared_attributes.named.contains_key(&attribute_name) {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::UnknownAttribute {
            name: attribute_name,
            typ_name: typ_name.to_string(),
          }
          .into_error(
            attribute.syntax().start_offset(),
            attribute.syntax().end_offset(),
          ),
        ),
        db,
      );
    } else if !seen.insert(attribute_name.clone()) {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::DuplicateAttribute {
            name: attribute_name,
            typ_name: typ_name.to_string(),
          }
          .into_error(
            attribute.syntax().start_offset(),
            attribute.syntax().end_offset(),
          ),
        ),
        db,
      );
    } else {
      seen.insert(attribute_name);
    }
  }
}

/// Validates that all nested element declarations use sub-element types declared by the parent type
fn check_sub_elements<'db>(
  db: &'db dyn Database,
  file: File,
  body: &BlockElementDeclarationBody,
  typ_name: &str,
  declared_subtypes: &BTreeMap<String, LazyTyp<'db>>,
) {
  for nested in body.element_declarations() {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    let Some(nested_typ) = element.typ() else {
      continue;
    };
    let nested_name = nested_typ.text().trim().to_string();
    let matches_subtype = declared_subtypes
      .keys()
      .any(|key| key.eq_ignore_ascii_case(&nested_name));

    if !matches_subtype {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::UnknownSubElement {
            name: nested_name,
            typ_name: typ_name.to_string(),
          }
          .into_error(nested.syntax().start_offset(), nested.syntax().end_offset()),
        ),
        db,
      );
    }

    validate_element_fields(db, file, nested.syntax());
  }
}

/// Validates that all settings on the element are declared by its type and not duplicated
fn check_element_settings<'db>(
  db: &'db dyn Database,
  element: &ElementDeclaration,
  typ_name: &str,
  declared_settings: &BTreeMap<String, LazyTyp<'db>>,
  unique_settings: &BTreeSet<String>,
) {
  let Some(setting_list) = element.setting_list() else {
    return;
  };
  let mut seen: HashSet<String> = HashSet::new();
  for item in setting_list.items() {
    let Some(name_node) = item.name() else {
      continue;
    };
    let setting_name = name_node.name();
    if !declared_settings.contains_key(&setting_name) {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::UnknownSetting {
            name: setting_name,
            typ_name: typ_name.to_string(),
          }
          .into_error(item.syntax().start_offset(), item.syntax().end_offset()),
        ),
        db,
      );
    } else if unique_settings.contains(&setting_name) && !seen.insert(setting_name.clone()) {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::DuplicateSetting {
            name: setting_name,
            typ_name: typ_name.to_string(),
          }
          .into_error(item.syntax().start_offset(), item.syntax().end_offset()),
        ),
        db,
      );
    } else {
      seen.insert(setting_name);
    }
  }
}

/// Validates arg counts, CSV usage, and duplicate names across all field lines in the body
fn check_field_lines<'db>(
  db: &'db dyn Database,
  file: File,
  body: &BlockElementDeclarationBody,
  declared_field_nested_typs: &BTreeMap<String, LazyTyp<'db>>,
) {
  if declared_field_nested_typs.is_empty() {
    return;
  }

  // Track first occurrence of each declared field name (name -> (start, end))
  let mut seen_names: HashMap<String, (usize, usize)> = HashMap::new();

  // Single field type: applies implicitly; multiple types: args[0] is the discriminator
  let is_single_typ = declared_field_nested_typs.len() == 1;

  for field_line in body.fields() {
    let args: Vec<_> = field_line.args().collect();
    if args.is_empty() {
      continue;
    }

    let (field_typ, arg_offset, typ_name) = if is_single_typ {
      let (key, lazy_typ) = declared_field_nested_typs.iter().next().unwrap();
      let Some(Typ::Element(element_typ)) = lazy_typ.resolve(db) else {
        continue;
      };
      (element_typ, 0usize, key.clone())
    } else {
      let Some(discriminator) = args[0].expr().and_then(|expr| expr.as_name()) else {
        continue;
      };
      let Some((key, lazy_typ)) = declared_field_nested_typs
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(&discriminator))
      else {
        continue;
      };
      let Some(Typ::Element(element_typ)) = lazy_typ.resolve(db) else {
        continue;
      };
      (element_typ, 1usize, key.clone())
    };

    let arg_fields = field_typ.typ_element_arg_fields(db);
    if arg_fields.is_empty() {
      continue;
    }

    /* Validate that CsvExpr only appears in [csv]-annotated arg slots */
    for (index, arg) in args[arg_offset..].iter().enumerate() {
      let Some(expr) = arg.expr() else { continue };
      if CsvExpr::cast(expr.syntax().clone()).is_some() {
        let csv_allowed = arg_fields
          .get(&(index + arg_offset))
          .map(|field| field.has_csv)
          .unwrap_or(false);
        if !csv_allowed {
          let arg_name = arg_fields
            .get(&(index + arg_offset))
            .map(|field| field.name.clone())
            .unwrap_or_else(|| format!("{}", index + arg_offset));

          Diagnostics::accumulate(
            Diagnostics(
              DiagnosticKind::CsvArgNotAllowed {
                arg_name,
                typ_name: typ_name.clone(),
              }
              .into_error(arg.syntax().start_offset(), arg.syntax().end_offset()),
            ),
            db,
          );
        }
      }
    }

    let attribute_fields = field_typ.typ_element_attributes(db);
    let inline_attribute_count = if attribute_fields.rest.is_none() {
      args[arg_offset..]
        .iter()
        .filter(|arg| {
          arg
            .expr()
            .and_then(|element| element.as_name())
            .is_some_and(|name| attribute_fields.named.contains_key(&name))
        })
        .count()
    } else {
      0
    };
    let actual_count = args
      .len()
      .saturating_sub(arg_offset)
      .saturating_sub(inline_attribute_count);
    let has_variadic = arg_fields.values().any(|field| field.is_variadic);
    let non_variadic_count = arg_fields
      .values()
      .filter(|field| !field.is_variadic)
      .count();

    let start = field_line.syntax().start_offset();
    let end = field_line.syntax().end_offset();

    if actual_count < non_variadic_count {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::TooFewFieldLineArgs {
            typ_name: typ_name.clone(),
            min_expected: non_variadic_count,
            found: actual_count,
          }
          .into_error(start, end),
        ),
        db,
      );
    } else if !has_variadic && actual_count > non_variadic_count {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::TooManyFieldLineArgs {
            typ_name: typ_name.clone(),
            max_expected: non_variadic_count,
            found: actual_count,
          }
          .into_error(start, end),
        ),
        db,
      );
    }

    // Detect duplicate declared field names
    for symbol in declare_field_line_symbols(db, file, &field_line, declared_field_nested_typs) {
      let name = symbol.name(db).to_string();

      let field_start = field_line.syntax().start_offset();
      let field_end = field_line.syntax().end_offset();
      if let Some(&(first_start, first_end)) = seen_names.get(&name) {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::DuplicateFieldDeclaration {
              name: name.clone(),
              first_start,
              first_end,
            }
            .into_error(field_start, field_end),
          ),
          db,
        );
      } else {
        seen_names.insert(name, (field_start, field_end));
      }
    }
  }
}
