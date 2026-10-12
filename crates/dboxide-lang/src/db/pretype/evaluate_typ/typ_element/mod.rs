pub(super) mod types;

use salsa::Database;

use crate::ast::{
  AstNode, BlockElementDeclaration, BlockElementDeclarationBody, ElementAttributeDeclaration,
  ElementDeclaration, ElementFieldDeclaration, ElementFieldDeclarationArg, Expr, FuncDeclaration,
  ListExpr, PostfixExpr,
};
use crate::db::prepost::{TypBodyExtract, extract_typ_body};
use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::{SymbolKind, VirtualTypKind};
use crate::db::types::{AnyTyp, Diagnostics, File, FuncObj, FuncTyp, OnUsePolicy, Typ};
use crate::diagnostics::DiagnosticKind;
use crate::evaluate_typ::typ_element::types::{EvaluatedTyp, EvaluatedTypField};

use super::constraints::{
  extract_constraint_entry, validate_alias_has_primary_label, validate_constraint_entry,
  validate_contiguous_arg_index, validate_declaration_field_labels,
  validate_field_annotation_consistency, validate_no_disallowed_nested_elements,
};
use super::types::{ArgPosition, FieldAnnotationFlags, LabelFlags};
use crate::db::prepost::evaluate_node::evaluate_lazy_typ_pretype;

/// Evaluate `type Name [element] { ... }` into an ElementTyp
pub fn evaluate_typ_element<'db>(
  db: &'db dyn Database,
  file: File,
  name: &str,
  block_declaration: &BlockElementDeclaration,
) -> Typ<'db> {
  let body = match block_declaration.body() {
    Some(body) => extract_typ_body(&body),
    None => TypBodyExtract::default(),
  };

  // validate setting annotations on the type declaration
  let mut evaluated = EvaluatedTyp {
    typ_settings: process_typ_settings(db, block_declaration),
    ..Default::default()
  };

  // 1. field declarations: `name Type [annotations]`
  process_typ_fields(db, file, &body.field_declarations, &mut evaluated);

  // 2. reject colon-syntax (`name: Type`) in type bodies
  disallow_colon_syntax(db, &body.attributes);

  // 3. nested type definitions (`type Name [setting] { ... }`)
  process_nested_types(db, file, &body.typ_declarations, &mut evaluated);
  validate_no_disallowed_nested_elements(db, &body.disallowed_elements);

  // 4. validate contiguous arg indices across all fields
  validate_contiguous_arg_index(
    db,
    evaluated
      .typ_field_declarations
      .iter()
      .map(|(key, value)| (key.as_str(), value)),
  );

  // 4b. validate that alias labels have a corresponding primary label
  validate_alias_has_primary_label(
    db,
    evaluated
      .typ_field_declarations
      .iter()
      .map(|(key, value)| (key.as_str(), value)),
  );

  // 5. namespace block (field re-exports and getters)
  process_namespace_block(db, file, &body.namespace_body, &mut evaluated);

  // 6. member functions
  process_member_functions(db, file, &body.func_declarations, &mut evaluated);

  // 7. constraints block
  process_constraints_block(db, &body.constraints, &mut evaluated);

  // 8. construct final ElementTyp
  evaluated.into_element_typ(db, name.to_string())
}

/* Type setting */

fn process_typ_settings(
  db: &dyn Database,
  block_declaration: &BlockElementDeclaration,
) -> types::EvaluatedTypSettings {
  let mut result = types::EvaluatedTypSettings::default();

  let Some(declaration) = ElementDeclaration::cast(block_declaration.syntax().clone()) else {
    return result;
  };
  let Some(settings) = declaration.setting_list() else {
    return result;
  };

  for item in settings.items() {
    let Some(name_node) = item.name() else {
      continue;
    };
    let setting = name_node.name();
    match setting.as_str() {
      "element" => {
        result.is_element_typ = true;
      }
      "field" => {
        result.is_field_typ = true;
      }
      "on use" => {
        let value = item
          .value()
          .and_then(|v| v.expr())
          .and_then(|e| e.as_name())
          .unwrap_or_default();
        match value.as_str() {
          "expand" => result.on_use = OnUsePolicy::Expand,
          _ => {
            let start = item.syntax().offset();
            salsa::Accumulator::accumulate(
              Diagnostics(
                DiagnosticKind::UnrecognizedOnUseValue { value }
                  .into_error(start, start + item.syntax().text_len()),
              ),
              db,
            );
          }
        }
      }
      _ => {
        let start = item.syntax().offset();
        salsa::Accumulator::accumulate(
          Diagnostics(
            DiagnosticKind::UnrecognizedTypSetting {
              setting: setting.to_string(),
            }
            .into_error(start, start + item.syntax().text_len()),
          ),
          db,
        );
      }
    }
  }

  if result.is_element_typ && result.is_field_typ {
    let start = settings.syntax().offset();
    salsa::Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::ConflictingTypRoles.into_error(start, start + settings.syntax().text_len()),
      ),
      db,
    );
  }

  result
}

/* Type field */

// `name Type [annotations]`
fn process_typ_fields<'db>(
  db: &'db dyn Database,
  file: File,
  field_declarations: &[ElementFieldDeclaration],
  evaluated: &mut EvaluatedTyp<'db>,
) {
  for field in field_declarations {
    let mut args = field.args();
    let Some(first_arg) = args.next() else {
      continue;
    };

    let Some((field_name, is_rest)) = parse_field_name(db, field, first_arg) else {
      continue;
    };
    let field_typ = parse_field_typ(db, file, &field_name, args.next());

    if is_rest {
      if evaluated.typ_rest_attribute.is_some() {
        let start = field.syntax().offset();
        salsa::Accumulator::accumulate(
          Diagnostics(
            DiagnosticKind::DuplicateRestAttribute
              .into_error(start, start + field.syntax().text_len()),
          ),
          db,
        );
      } else {
        evaluated.typ_rest_attribute = Some(field_typ);
      }
      continue;
    }

    let annotations = parse_field_annotations(field);
    let label = resolve_label(db, annotations.label, field_typ);

    let declaration = EvaluatedTypField {
      typ: field_typ,
      arg_position: annotations.arg_position,
      flags: annotations.flags,
      label,
      setting_key: annotations.setting_key,
      node: field.syntax().clone(),
    };

    validate_declaration_field_labels(db, &field_name, &declaration);
    validate_field_annotation_consistency(db, &field_name, &declaration, annotations.label);
    evaluated
      .typ_field_declarations
      .insert(field_name, declaration);
  }
}

fn parse_field_name(
  db: &dyn Database,
  field: &ElementFieldDeclaration,
  first_arg: ElementFieldDeclarationArg,
) -> Option<(String, bool)> {
  if let Some(list) = first_arg
    .expr()
    .and_then(|e| ListExpr::cast(e.syntax().clone()))
  {
    // rest attribute: `[name]` - exactly one ident inside brackets
    let mut items = list.items();
    let first_item = items.next().and_then(|item| item.as_name());
    if items.next().is_some() {
      let start = field.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::MalformedRestAttribute
            .into_error(start, start + field.syntax().text_len()),
        ),
        db,
      );
      return None;
    }
    first_item.map(|name| (name, true))
  } else {
    first_arg
      .expr()
      .and_then(|e| e.as_name())
      .map(|name| (name, false))
  }
}

fn parse_field_typ<'db>(
  db: &'db dyn Database,
  file: File,
  field_name: &str,
  type_arg: Option<ElementFieldDeclarationArg>,
) -> LazyTyp<'db> {
  let resolved = type_arg
    .as_ref()
    .and_then(|arg| arg.expr())
    .and_then(|expr| evaluate_lazy_typ_pretype(db, file, &expr));

  if resolved.is_none()
    && let Some(arg) = &type_arg
  {
    let start = arg.syntax().offset();
    salsa::Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::InvalidFieldTyp {
          field_name: field_name.to_string(),
        }
        .into_error(start, start + arg.syntax().text_len()),
      ),
      db,
    );
  }

  resolved.unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()))
}

struct ParsedAnnotations {
  arg_position: Option<ArgPosition>,
  flags: FieldAnnotationFlags,
  label: LabelFlags,
  setting_key: Option<Option<String>>,
}

fn parse_field_annotations(field: &ElementFieldDeclaration) -> ParsedAnnotations {
  let mut result = ParsedAnnotations {
    arg_position: None,
    flags: FieldAnnotationFlags::empty(),
    label: LabelFlags::empty(),
    setting_key: None,
  };

  let Some(setting_list) = field.setting_list() else {
    return result;
  };

  for item in setting_list.items() {
    let Some(item_name) = item.name() else {
      continue;
    };
    match item_name.name().as_str() {
      "attribute" => result.flags |= FieldAnnotationFlags::ATTRIBUTE,
      "label" => result.label |= LabelFlags::LABEL,
      "label alias" => result.label |= LabelFlags::ALIAS,
      "unique" => result.label |= LabelFlags::UNIQUE,
      "csv" => result.flags |= FieldAnnotationFlags::CSV,
      "arg" => {
        if let Some(value_expr) = item.value().and_then(|value| value.expr()) {
          result.arg_position = parse_positional_index(&value_expr).map(|(n, variadic)| {
            if variadic {
              ArgPosition::Variadic(n)
            } else {
              ArgPosition::Exact(n)
            }
          });
        }
      }
      "setting" => {
        result.setting_key = Some(
          item
            .value()
            .and_then(|v| v.expr())
            .and_then(|e| e.as_name()),
        );
      }
      _ => {}
    }
  }

  result
}

fn resolve_label<'db>(
  db: &'db dyn Database,
  raw_label: LabelFlags,
  field_typ: LazyTyp<'db>,
) -> Option<LabelFlags> {
  if !raw_label.intersects(LabelFlags::LABEL | LabelFlags::ALIAS) {
    return None;
  }
  let mut stored = raw_label & (LabelFlags::ALIAS | LabelFlags::UNIQUE);
  if is_qualified_declaration(db, field_typ) {
    stored |= LabelFlags::EXPOSES_NAMESPACE;
  }
  Some(stored)
}

// `name: Type` - colon-syntax is never valid in a type body
fn disallow_colon_syntax(db: &dyn Database, attributes: &[ElementAttributeDeclaration]) {
  for attribute in attributes {
    let start = attribute.syntax().offset();
    let field_name = match attribute
      .name()
      .and_then(|n| n.name())
      .and_then(|e| e.as_name())
    {
      Some(name) => name,
      None => continue,
    };
    salsa::Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::ColonDeclarationInTypBody { field_name }
          .into_error(start, start + attribute.syntax().text_len()),
      ),
      db,
    );
  }
}

// `type Name [setting] { ... }`
fn process_nested_types<'db>(
  db: &'db dyn Database,
  file: File,
  typ_declarations: &[BlockElementDeclaration],
  evaluated: &mut EvaluatedTyp<'db>,
) {
  for nested_type in typ_declarations {
    let Some(declaration) = ElementDeclaration::cast(nested_type.syntax().clone()) else {
      continue;
    };
    let Some(declaration_name) = declaration.declaration_name() else {
      continue;
    };
    let has_element = declaration.setting_list().is_some_and(|settings| {
      settings.items().any(|item| {
        item
          .name()
          .is_some_and(|name| name.name().eq_ignore_ascii_case("element"))
      })
    });
    let has_field = declaration.setting_list().is_some_and(|settings| {
      settings.items().any(|item| {
        item
          .name()
          .is_some_and(|name| name.name().eq_ignore_ascii_case("field"))
      })
    });

    if !has_element && !has_field {
      let start = nested_type.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::NestedTypMissingRole {
            typ_name: declaration_name.clone(),
          }
          .into_error(start, start + nested_type.syntax().text_len()),
        ),
        db,
      );
      continue;
    }

    let nested_typ = evaluate_typ_element(db, file, &declaration_name, nested_type);
    if has_element {
      evaluated
        .typ_nested_typs
        .element_nested_typs
        .insert(declaration_name, LazyTyp::eager(nested_typ));
    } else {
      evaluated
        .typ_nested_typs
        .field_nested_typs
        .insert(declaration_name, LazyTyp::eager(nested_typ));
    }
  }
}

// `namespace { name | get name(): Type { } }`
fn process_namespace_block<'db>(
  db: &'db dyn Database,
  file: File,
  namespace_body: &Option<BlockElementDeclarationBody>,
  evaluated: &mut EvaluatedTyp<'db>,
) {
  let Some(body) = namespace_body else { return };

  // Field re-exports: `namespace { name }`, re-exports the type of a declared field
  for field in body.fields() {
    let Some(name_arg) = field.args().next() else {
      continue;
    };
    let Some(field_name) = name_arg.expr().and_then(|expr| expr.as_name()) else {
      continue;
    };
    if let Some(declaration) = evaluated.typ_field_declarations.get(&field_name) {
      evaluated.typ_namespace.insert(field_name, declaration.typ);
    } else {
      let start = field.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::UnknownNamespaceExport { field_name }
            .into_error(start, start + field.syntax().text_len()),
        ),
        db,
      );
    }
  }

  // Colon-syntax in namespace block is not a valid form
  for attribute in body.attributes() {
    let start = attribute.syntax().offset();
    let field_name = match attribute
      .name()
      .and_then(|n| n.name())
      .and_then(|e| e.as_name())
    {
      Some(name) => name,
      None => continue,
    };
    salsa::Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::ColonDeclarationInNamespaceBlock { field_name }
          .into_error(start, start + attribute.syntax().text_len()),
      ),
      db,
    );
  }

  // Getters: `get name(): Type { }`
  // computed property, exposed in both namespace and vtable
  for getter in body.get_declarations() {
    let Some(name_node) = getter.name() else {
      continue;
    };
    let getter_name = name_node.text().trim().to_string();
    let typ = getter
      .return_typ()
      .and_then(|return_typ| return_typ.type_expr())
      .and_then(|expr| evaluate_lazy_typ_pretype(db, file, &expr))
      .unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()));
    evaluated.typ_namespace.insert(getter_name.clone(), typ);

    let signature = FuncSignature::new(db, vec![], typ);
    let func_obj = FuncObj::new(
      db,
      getter_name.clone(),
      FuncTyp::new(db, signature),
      Some(getter.syntax().clone()),
      None,
    );
    evaluated.vtable.insert(getter_name, func_obj);
  }
}

// `fn name(params): ReturnType { ... }`
fn process_member_functions<'db>(
  db: &'db dyn Database,
  file: File,
  function_declarations: &[FuncDeclaration],
  evaluated: &mut EvaluatedTyp<'db>,
) {
  for func_declaration in function_declarations {
    let Some(name_node) = func_declaration.name() else {
      continue;
    };
    let func_name = name_node.text().trim().to_string();
    // params and ret are lazy - symbol references resolved later, not at pretype
    let ret_typ = func_declaration
      .return_typ()
      .and_then(|rt| rt.type_expr())
      .and_then(|expr| evaluate_lazy_typ_pretype(db, file, &expr))
      .unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()));

    let mut param_types = Vec::new();
    if let Some(params_node) = func_declaration.params() {
      for param in params_node.params() {
        let param_typ = param
          .type_expr()
          .and_then(|expr| evaluate_lazy_typ_pretype(db, file, &expr))
          .unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()));
        param_types.push(param_typ);
      }
    }

    let sig = FuncSignature::new(db, param_types, ret_typ);
    let func_obj = FuncObj::new(
      db,
      func_name.clone(),
      FuncTyp::new(db, sig),
      Some(func_declaration.syntax().clone()),
      None,
    );
    evaluated.vtable.insert(func_name, func_obj);
  }
}

// Parse `n` or `n..` from a setting value expression
fn parse_positional_index(expr: &Expr) -> Option<(usize, bool)> {
  if let Some(postfix) = PostfixExpr::cast(expr.syntax().clone())
    && postfix.op().as_deref() == Some("..")
  {
    let n = postfix.operand()?.as_int()?;
    if n >= 0 {
      return Some((n as usize, true));
    }
  }
  let n = expr.as_int()?;
  if n >= 0 {
    Some((n as usize, false))
  } else {
    None
  }
}

fn is_qualified_declaration<'db>(db: &'db dyn Database, typ: LazyTyp<'db>) -> bool {
  use crate::db::types::Typ;
  matches!(typ, LazyTyp::Eager(Typ::QualifiedDeclaration(_)))
    || matches!(
      typ,
      LazyTyp::Lazy(symbol) if matches!(
        symbol.kind(db),
        SymbolKind::VirtualTyp(_, VirtualTypKind::MetatypeQualifiedDeclaration)
      )
    )
}

// `constraints { name: closure }`
fn process_constraints_block(
  db: &dyn Database,
  constraints_block: &Option<BlockElementDeclaration>,
  evaluated: &mut EvaluatedTyp<'_>,
) {
  if let Some(block) = constraints_block
    && let Some(body) = block.body()
  {
    for attribute in body.attributes() {
      if let Some(entry) = extract_constraint_entry(&attribute) {
        let (_, constraint_def) = validate_constraint_entry(db, &entry);
        if let Some(definition) = constraint_def {
          evaluated
            .typ_constraints
            .insert(definition.name.clone(), definition);
        }
      }
    }
  }
}
