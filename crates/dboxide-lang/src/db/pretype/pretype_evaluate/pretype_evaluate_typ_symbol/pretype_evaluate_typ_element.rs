use salsa::Database;

use crate::ast::{
  AstNode, BlockElementDeclaration, BlockElementDeclarationBody, ElementAttributeDeclaration,
  ElementDeclaration, ElementFieldDeclaration, Expr, FuncDeclaration, ListExpr, PostfixExpr,
};
use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::{SymbolKind, VirtualTypKind};
use crate::db::types::{AnyTyp, Diagnostics, File, FuncObj, FuncTyp, OnUsePolicy, Typ};
use crate::db::utils::{TypBodyExtract, extract_typ_body};
use crate::diagnostics::Diagnostic;

use super::super::pretype_evaluate_node::{pretype_evaluate_lazy_typ, pretype_evaluate_node};
use super::constraints::{
  extract_constraint_entry, validate_constraint_entry, validate_contiguous_arg_index,
  validate_declaration_field_labels, validate_field_annotation_consistency,
  validate_no_disallowed_nested_elements,
};
use super::types::{
  ArgPosition, EvaluatedAttributeDeclaration, EvaluatedFieldDeclaration, EvaluatedNestedElement,
  EvaluatedTyp, FieldAnnotationFlags, LabelFlags,
};

/// Evaluate `type Name [element] { ... }` into an ElementTyp
pub fn pretype_evaluate_typ_element_node<'db>(
  db: &'db dyn Database,
  file: File,
  name: &str,
  block_declaration: &BlockElementDeclaration,
) -> Typ<'db> {
  let body = match block_declaration.body() {
    Some(body) => extract_typ_body(&body),
    None => TypBodyExtract::default(),
  };

  let on_use = ElementDeclaration::cast(block_declaration.syntax().clone())
    .and_then(|decl| decl.setting_list())
    .and_then(|settings| settings.get_item("on use"))
    .and_then(|item| item.value()?.expr()?.as_name())
    // TODO: emit diagnostic for unrecognized on_use values (e.g. "inline")
    .map(|v| {
      if v == "expand" {
        OnUsePolicy::Expand
      } else {
        OnUsePolicy::Default
      }
    })
    .unwrap_or(OnUsePolicy::Default);

  let mut evaluated = EvaluatedTyp {
    on_use,
    ..EvaluatedTyp::default()
  };

  // Step 1: Process positional field declarations
  process_field_declarations(db, file, &body.field_declarations, &mut evaluated);

  // Step 2: Process key-value attribute declarations (@attr or attr: type)
  process_attribute_declarations(db, file, &body.attributes, &mut evaluated);

  // Step 3: Process nested type definitions (field subtypes vs element subtypes)
  process_nested_types(db, file, &body.typ_declarations, &mut evaluated);

  // Step 3b: Validate non-type nested elements
  let disallowed: Vec<EvaluatedNestedElement> = body
    .disallowed_elements
    .iter()
    .map(|element| {
      let element_typ = ElementDeclaration::cast(element.syntax().clone())
        .and_then(|e| e.typ())
        .map(|t| t.text().trim().to_string())
        .unwrap_or_else(|| "<unknown>".to_string());
      EvaluatedNestedElement {
        element_typ,
        node: element.syntax().clone(),
      }
    })
    .collect();
  validate_no_disallowed_nested_elements(db, &disallowed);

  // Step 3c: Batch validate contiguous arg indices
  validate_contiguous_arg_index(
    db,
    evaluated
      .field_declarations
      .iter()
      .map(|(k, v)| (k.as_str(), v)),
  );

  // Step 4: Process namespace block (exposed fields and getters)
  process_namespace_block(db, file, &body.namespace_body, &mut evaluated);

  // Step 5: Process member functions
  process_member_functions(db, file, &body.function_declarations, &mut evaluated);

  // Step 6: Process constraints block (named validation rules)
  process_constraints_block(db, &body.constraints, &mut evaluated);

  // Step 7: Construct final ElementTyp
  evaluated.into_element_typ(db, name.to_string())
}

fn process_field_declarations<'db>(
  db: &'db dyn Database,
  file: File,
  field_declarations: &[ElementFieldDeclaration],
  evaluated: &mut EvaluatedTyp<'db>,
) {
  for field in field_declarations {
    let mut args = field.args();

    let first_arg = match args.next() {
      Some(a) => a,
      None => continue,
    };

    // rest attribute syntax: `[name] Type` - first arg is a ListExpr with a single ident
    let field_name = if let Some(list) = first_arg
      .expr()
      .and_then(|e| ListExpr::cast(e.syntax().clone()))
    {
      let mut items = list.items();
      let first_item = items.next().and_then(|item| item.as_name());
      let has_extra = items.next().is_some();
      if has_extra {
        let start = field.syntax().offset();
        salsa::Accumulator::accumulate(
          Diagnostics(Diagnostic::MalformedRestAttribute {
            start_offset: start,
            end_offset: start + field.syntax().text_len(),
          }),
          db,
        );
        continue;
      }
      match first_item {
        Some(n) => n,
        None => continue,
      }
    } else {
      match first_arg.expr().and_then(|e| e.as_name()) {
        Some(n) => n,
        None => continue,
      }
    };

    let type_arg = args.next();
    let field_typ = type_arg
      .as_ref()
      .and_then(|arg| arg.expr())
      .and_then(|expr| pretype_evaluate_lazy_typ(db, file, &expr));

    if field_typ.is_none()
      && let Some(arg) = &type_arg
    {
      let start = arg.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(Diagnostic::InvalidFieldType {
          field_name: field_name.clone(),
          start_offset: start,
          end_offset: start + arg.syntax().text_len(),
        }),
        db,
      );
    }

    let field_typ = field_typ.unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()));

    let mut arg_position = None;
    let mut field_flags = FieldAnnotationFlags::empty();
    let mut raw_label = LabelFlags::empty();
    let mut setting_key: Option<Option<String>> = None;

    if let Some(setting_list) = field.setting_list() {
      for item in setting_list.items() {
        let Some(item_name) = item.name() else {
          continue;
        };
        match item_name.name().as_str() {
          "attribute" => field_flags |= FieldAnnotationFlags::ATTRIBUTE,
          "label" => raw_label |= LabelFlags::LABEL,
          "label alias" => raw_label |= LabelFlags::ALIAS,
          "unique" => raw_label |= LabelFlags::UNIQUE,
          "csv" => field_flags |= FieldAnnotationFlags::CSV,
          "arg" => {
            if let Some(value_expr) = item.value().and_then(|v| v.expr()) {
              arg_position = parse_positional_index(&value_expr).map(|(n, variadic)| {
                if variadic {
                  ArgPosition::Variadic(n)
                } else {
                  ArgPosition::Exact(n)
                }
              });
            }
          }
          "setting" => {
            let key = item
              .value()
              .and_then(|v| v.expr())
              .and_then(|e| e.as_name());
            setting_key = Some(key);
          }
          _ => {}
        }
      }
    }

    let label = if raw_label.intersects(LabelFlags::LABEL | LabelFlags::ALIAS) {
      let mut stored = raw_label & (LabelFlags::ALIAS | LabelFlags::UNIQUE);
      if is_qualified_declaration(db, field_typ) {
        stored |= LabelFlags::EXPOSES_NAMESPACE;
      }
      Some(stored)
    } else {
      None
    };

    let declaration = EvaluatedFieldDeclaration {
      typ: field_typ,
      arg_position,
      flags: field_flags,
      label,
      setting_key,
      node: field.syntax().clone(),
    };

    validate_declaration_field_labels(db, &field_name, &declaration);
    validate_field_annotation_consistency(db, &field_name, &declaration, raw_label);
    evaluated.field_declarations.insert(field_name, declaration);
  }
}

/// Colon-syntax `name: type` declarations are implicitly `[attribute]`
fn process_attribute_declarations<'db>(
  db: &'db dyn Database,
  file: File,
  attributes: &[ElementAttributeDeclaration],
  evaluated: &mut EvaluatedTyp<'db>,
) {
  for attr in attributes {
    let Some(attr_name) = attr
      .name()
      .and_then(|name| name.name())
      .and_then(|expr| expr.as_name())
    else {
      continue;
    };
    let value_arg = attr.value().and_then(|value| value.expr());
    let attr_typ = value_arg
      .as_ref()
      .and_then(|expr| pretype_evaluate_lazy_typ(db, file, expr));

    if attr_typ.is_none()
      && let Some(arg) = &value_arg
    {
      let start = arg.syntax().offset();
      salsa::Accumulator::accumulate(
        Diagnostics(Diagnostic::InvalidFieldType {
          field_name: attr_name.clone(),
          start_offset: start,
          end_offset: start + arg.syntax().text_len(),
        }),
        db,
      );
    }

    let attr_typ = attr_typ.unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()));

    evaluated
      .attribute_declarations
      .insert(attr_name, EvaluatedAttributeDeclaration { typ: attr_typ });
  }
}

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
    let is_element_role = declaration.setting_list().is_some_and(|settings| {
      settings.items().any(|item| {
        item
          .name()
          .is_some_and(|name| name.name().eq_ignore_ascii_case("element"))
      })
    });
    let nested_typ = pretype_evaluate_typ_element_node(db, file, &declaration_name, nested_type);
    if is_element_role {
      evaluated
        .element_subtypes
        .insert(declaration_name, LazyTyp::eager(nested_typ));
    } else {
      evaluated
        .field_subtypes
        .insert(declaration_name, LazyTyp::eager(nested_typ));
    }
  }
}

fn process_namespace_block<'db>(
  db: &'db dyn Database,
  file: File,
  namespace_body: &Option<BlockElementDeclarationBody>,
  evaluated: &mut EvaluatedTyp<'db>,
) {
  let Some(body) = namespace_body else { return };

  for field in body.fields() {
    let Some(name_arg) = field.args().next() else {
      continue;
    };
    let Some(field_name) = name_arg.expr().and_then(|expr| expr.as_name()) else {
      continue;
    };
    if let Some(decl) = evaluated.field_declarations.get(&field_name) {
      evaluated.namespace.insert(field_name, decl.typ);
    } else if let Some(attr) = evaluated.attribute_declarations.get(&field_name) {
      evaluated.namespace.insert(field_name, attr.typ);
    }
  }

  for getter in body.get_declarations() {
    let Some(name_node) = getter.name() else {
      continue;
    };
    let getter_name = name_node.text().trim().to_string();
    let typ = getter
      .return_typ()
      .and_then(|rt| rt.type_expr())
      .and_then(|expr| pretype_evaluate_lazy_typ(db, file, &expr))
      .unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()));
    evaluated.namespace.insert(getter_name.clone(), typ);

    let ret_typ = typ.as_eager().unwrap_or_else(|| AnyTyp::new(db).into());
    let signature = FuncSignature::new(db, vec![], ret_typ);
    let func_obj = FuncObj::new(
      db,
      getter_name.clone(),
      FuncTyp::new(db, signature),
      getter.body().map(|b| b.syntax().clone()),
    );
    evaluated.vtable.insert(getter_name, func_obj);
  }
}

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
    let ret_typ = func_declaration
      .return_typ()
      .and_then(|rt| rt.type_expr())
      .and_then(|expr| pretype_evaluate_node(db, file, &expr))
      .and_then(|obj| obj.as_typ())
      .unwrap_or_else(|| AnyTyp::new(db).into());

    let mut param_types = Vec::new();
    if let Some(params_node) = func_declaration.params() {
      for param in params_node.params() {
        let param_typ = param
          .type_expr()
          .and_then(|expr| pretype_evaluate_node(db, file, &expr))
          .and_then(|obj| obj.as_typ())
          .unwrap_or_else(|| AnyTyp::new(db).into());
        param_types.push(param_typ);
      }
    }

    let sig = FuncSignature::new(db, param_types, ret_typ);
    let func_obj = FuncObj::new(
      db,
      func_name.clone(),
      FuncTyp::new(db, sig),
      func_declaration.body().map(|b| b.syntax().clone()),
    );
    evaluated.vtable.insert(func_name, func_obj);
  }
}

/// Parse `n` or `n..` from a setting value expression
/// Returns `(index, is_variadic)` or `None` if not a valid non-negative integer
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
      LazyTyp::Lazy(sym) if matches!(
        sym.kind(db),
        SymbolKind::VirtualTyp(_, VirtualTypKind::MetatypeQualifiedDeclaration)
      )
    )
}

fn process_constraints_block(
  db: &dyn Database,
  constraints_block: &Option<BlockElementDeclaration>,
  evaluated: &mut EvaluatedTyp<'_>,
) {
  if let Some(block) = constraints_block
    && let Some(body) = block.body()
  {
    for attr in body.attributes() {
      if let Some(entry) = extract_constraint_entry(&attr) {
        let (_, constraint_def) = validate_constraint_entry(db, &entry);
        if let Some(def) = constraint_def {
          evaluated.constraints.insert(def.name.clone(), def);
        }
      }
    }
  }
}
