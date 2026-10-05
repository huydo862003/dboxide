use std::collections::BTreeMap;

use salsa::Database;

use crate::ast::{
  AstNode, BlockElementDeclaration, BlockElementDeclarationBody, ElementAttributeDeclaration,
  ElementDeclaration, ElementFieldDeclaration, FnDeclaration,
};
use crate::db::pretype::comp_evaluate::{comp_evaluate_lazy_typ, comp_evaluate_node};
use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::{SymbolKind, VirtualTypKind};
use crate::db::types::{
  AnyTyp, ConstraintDefinition, ElementTyp, File, FuncObj, FuncTyp, LabelDefinition, Typ,
};
use crate::db::utils::{TypBodyExtract, extract_typ_body};

use super::constraints::{
  validate_constraint_entry, validate_contiguous_arg_index, validate_declaration_field_labels,
};

/// Evaluated positional fields with labels, attributes, and settings
#[derive(Default)]
struct EvaluatedFields<'db> {
  fields: BTreeMap<String, LazyTyp<'db>>,
  labels: Vec<LabelDefinition>,
  attributes: BTreeMap<String, LazyTyp<'db>>,
  settings: BTreeMap<String, LazyTyp<'db>>,
}

/// Evaluated nested element sub-types and field sub-types
#[derive(Default)]
struct EvaluatedSubtypes<'db> {
  field_subtypes: BTreeMap<String, LazyTyp<'db>>,
  element_subtypes: BTreeMap<String, LazyTyp<'db>>,
}

/// Evaluate `type Name [element] { ... }` into an ElementTyp
pub fn evaluate_typ_element_node<'db>(
  db: &'db dyn Database,
  file: File,
  name: &str,
  block_declaration: &BlockElementDeclaration,
) -> Typ<'db> {
  let body = match block_declaration.body() {
    Some(body) => extract_typ_body(&body),
    None => TypBodyExtract::default(),
  };

  // Step 1: Process positional field declarations and settings
  let mut evaluated_fields = process_field_declarations(db, file, &body.field_declarations);

  // Step 2: Process key-value attribute declarations (@attr or attr: type)
  process_attribute_declarations(
    db,
    file,
    &body.attributes,
    &mut evaluated_fields.fields,
    &mut evaluated_fields.attributes,
  );

  // Step 3: Process nested type definitions (field subtypes vs element subtypes)
  let evaluated_subtypes = process_nested_types(db, file, &body.typ_declarations);

  // Step 4: Process namespace block (exposed fields and getters)
  let mut vtable = BTreeMap::new();
  let namespace = process_namespace_block(
    db,
    file,
    &body.namespace_body,
    &evaluated_fields.fields,
    &mut vtable,
  );

  // Step 5: Process member functions
  process_member_functions(db, file, &body.function_declarations, &mut vtable);

  // Step 6: Process constraints block (named validation rules)
  let constraints = process_constraints_block(db, &body.constraints);

  // Step 7: Construct final ElementTyp with all evaluated members
  ElementTyp::new(
    db,
    name.to_string(),
    evaluated_fields.fields,
    evaluated_fields.labels,
    evaluated_fields.attributes,
    evaluated_fields.settings,
    evaluated_subtypes.field_subtypes,
    evaluated_subtypes.element_subtypes,
    namespace,
    vtable,
    constraints,
    None,
  )
  .into()
}

/// Process positional field declarations, labels, settings, and argument indices
fn process_field_declarations<'db>(
  db: &'db dyn Database,
  file: File,
  field_declarations: &[ElementFieldDeclaration],
) -> EvaluatedFields<'db> {
  let mut evaluated = EvaluatedFields::default();
  let mut expected_arg_index = 0;

  for field in field_declarations {
    let mut args = field.args();

    let Some(field_name) = args
      .next()
      .and_then(|name_arg| name_arg.expr())
      .and_then(|expr| expr.as_name())
    else {
      continue;
    };

    let field_typ = args
      .next()
      .and_then(|arg| arg.expr())
      .and_then(|expr| comp_evaluate_lazy_typ(db, file, &expr))
      .unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()));

    evaluated.fields.insert(field_name.clone(), field_typ);

    let mut is_label = false;
    let mut is_alias = false;
    let mut unique = false;
    let mut is_attribute = false;
    let mut is_setting = false;
    let mut arg_index = None;
    let mut has_qualified_alias = false;

    if let Some(setting_list) = field.setting_list() {
      for item in setting_list.items() {
        if let Some(item_name) = item.name() {
          match item_name.name().as_str() {
            "label" => is_label = true,
            "label alias" => {
              is_label = true;
              is_alias = true;
            }
            "unique" => unique = true,
            "attribute" => is_attribute = true,
            "setting" => is_setting = true,
            "arg" => {
              if let Some(value) = item
                .value()
                .and_then(|value| value.expr())
                .and_then(|expr| expr.as_int())
                && value >= 0
              {
                arg_index = Some(value as usize);
              }
            }
            _ => {}
          }
        }
      }

      has_qualified_alias = setting_list.has_item("qualified alias");

      if is_label {
        let exposes_namespace = match field_typ {
          LazyTyp::Eager(Typ::QualifiedDeclaration(_)) => true,
          LazyTyp::Lazy(symbol) => matches!(
            symbol.kind(db),
            SymbolKind::VirtualTyp(_, VirtualTypKind::MetatypeQualifiedDeclaration)
          ),
          _ => false,
        };
        evaluated.labels.push(LabelDefinition {
          field_name: field_name.clone(),
          is_alias,
          unique,
          exposes_namespace,
        });
      }

      if is_attribute {
        evaluated.attributes.insert(field_name.clone(), field_typ);
      }

      if is_setting {
        evaluated.settings.insert(field_name.clone(), field_typ);
      }
    }

    validate_declaration_field_labels(
      db,
      field,
      &field_name,
      field_typ,
      is_label,
      has_qualified_alias,
    );
    validate_contiguous_arg_index(db, field, &field_name, arg_index, &mut expected_arg_index);
  }

  evaluated
}

/// Process key-value attribute declarations (@attr or attr: type)
fn process_attribute_declarations<'db>(
  db: &'db dyn Database,
  file: File,
  attributes: &[ElementAttributeDeclaration],
  fields: &mut BTreeMap<String, LazyTyp<'db>>,
  attr_map: &mut BTreeMap<String, LazyTyp<'db>>,
) {
  for attr in attributes {
    let Some(attr_name) = attr
      .name()
      .and_then(|name| name.name())
      .and_then(|expr| expr.as_name())
    else {
      continue;
    };
    let attr_typ = attr
      .value()
      .and_then(|value| value.expr())
      .and_then(|expr| comp_evaluate_lazy_typ(db, file, &expr))
      .unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()));
    fields.insert(attr_name.clone(), attr_typ);
    attr_map.insert(attr_name, attr_typ);
  }
}

/// Process nested type definitions (field subtypes vs element subtypes)
fn process_nested_types<'db>(
  db: &'db dyn Database,
  file: File,
  typ_declarations: &[BlockElementDeclaration],
) -> EvaluatedSubtypes<'db> {
  let mut evaluated = EvaluatedSubtypes::default();

  for nested_type in typ_declarations {
    let Some(declaration) = ElementDeclaration::cast(nested_type.syntax().clone()) else {
      continue;
    };
    let Some(declaration_name) = declaration.declaration_name() else {
      continue;
    };
    let is_element_role = declaration.setting_list().is_some_and(|setting_list| {
      setting_list.items().any(|item| {
        item
          .name()
          .is_some_and(|name| name.name().eq_ignore_ascii_case("element"))
      })
    });
    let nested_typ = evaluate_typ_element_node(db, file, &declaration_name, nested_type);
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

  evaluated
}

/// Process namespace block (exposed fields, static getters, and methods)
fn process_namespace_block<'db>(
  db: &'db dyn Database,
  file: File,
  namespace_body: &Option<BlockElementDeclarationBody>,
  fields: &BTreeMap<String, LazyTyp<'db>>,
  vtable: &mut BTreeMap<String, FuncObj<'db>>,
) -> BTreeMap<String, LazyTyp<'db>> {
  let mut namespace = BTreeMap::new();
  let Some(body) = namespace_body else {
    return namespace;
  };

  for field in body.fields() {
    let Some(name_arg) = field.args().next() else {
      continue;
    };
    let Some(field_name) = name_arg.expr().and_then(|expr| expr.as_name()) else {
      continue;
    };
    if let Some(typ) = fields.get(&field_name) {
      namespace.insert(field_name, *typ);
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
      .and_then(|expr| comp_evaluate_lazy_typ(db, file, &expr))
      .unwrap_or_else(|| LazyTyp::eager(AnyTyp::new(db).into()));
    namespace.insert(getter_name.clone(), typ);

    let ret_typ = typ.as_eager().unwrap_or_else(|| AnyTyp::new(db).into());
    let signature = FuncSignature::new(db, vec![], ret_typ);
    let func_typ = FuncTyp::new(db, signature);
    let func_obj = FuncObj::new(
      db,
      getter_name.clone(),
      func_typ,
      getter.body().map(|b| b.syntax().clone()),
    );
    vtable.insert(getter_name, func_obj);
  }

  namespace
}

/// Process member functions and add them to the element vtable
fn process_member_functions<'db>(
  db: &'db dyn Database,
  file: File,
  function_declarations: &[FnDeclaration],
  vtable: &mut BTreeMap<String, FuncObj<'db>>,
) {
  for func_declaration in function_declarations {
    let Some(name_node) = func_declaration.name() else {
      continue;
    };
    let func_name = name_node.text().trim().to_string();
    let ret_typ = func_declaration
      .return_typ()
      .and_then(|rt| rt.type_expr())
      .and_then(|expr| comp_evaluate_node(db, file, &expr))
      .and_then(|obj| obj.as_typ())
      .unwrap_or_else(|| AnyTyp::new(db).into());

    let mut param_types = Vec::new();
    if let Some(params_node) = func_declaration.params() {
      for param in params_node.params() {
        let param_typ = param
          .type_expr()
          .and_then(|expr| comp_evaluate_node(db, file, &expr))
          .and_then(|obj| obj.as_typ())
          .unwrap_or_else(|| AnyTyp::new(db).into());
        param_types.push(param_typ);
      }
    }

    let sig = FuncSignature::new(db, param_types, ret_typ);
    let func_typ = FuncTyp::new(db, sig);
    let func_obj = FuncObj::new(
      db,
      func_name.clone(),
      func_typ,
      func_declaration.body().map(|b| b.syntax().clone()),
    );
    vtable.insert(func_name, func_obj);
  }
}

/// Process constraints block (named validation rules)
fn process_constraints_block(
  db: &dyn Database,
  constraints_block: &Option<BlockElementDeclaration>,
) -> BTreeMap<String, ConstraintDefinition> {
  let mut constraints = BTreeMap::new();
  if let Some(block) = constraints_block
    && let Some(body) = block.body()
  {
    for attr in body.attributes() {
      if let Some(constraint_def) = validate_constraint_entry(db, &attr) {
        constraints.insert(constraint_def.name.clone(), constraint_def);
      }
    }
  }
  constraints
}
