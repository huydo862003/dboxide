pub(super) mod types;

use std::collections::BTreeMap;

use salsa::{Accumulator, Database};

use crate::ast::{
  AstNode, BlockElementDeclaration, BlockElementDeclarationBody, ElementAttributeDeclaration,
  ElementDeclaration, ElementFieldDeclaration, ElementFieldDeclarationArg, Expr, ListExpr,
  PostfixExpr, SettingList,
};
use crate::db::pretype_evaluate_typ::typ_element::types::{
  EvaluatedTyp, EvaluatedTypField, EvaluatedTypNestedTyps, EvaluatedTypSettings,
};
use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{
  Diagnostics, File, FuncObj, FuncTyp, ObjTyp, OnUsePolicy, Typ, TypConstraint, TypNamespace,
};
use crate::diagnostics::DiagnosticKind;

use super::constraints::{
  extract_constraint_entry, validate_alias_has_primary_label, validate_constraint_entry,
  validate_contiguous_arg_index, validate_declaration_field_labels,
  validate_field_annotation_consistency, validate_unique_label_fields,
};
use super::types::{ArgPosition, FieldAnnotationFlags};
use crate::db::pretype_evaluate_typ::evaluate_node::pretype_evaluate_lazy_typ;

/// Evaluate `type Name [element] { ... }` into an ElementTyp
pub fn pretype_evaluate_typ_element<'db>(
  db: &'db dyn Database,
  file: File,
  name: &str,
  block_declaration: &BlockElementDeclaration,
) -> Typ<'db> {
  /* Type header */
  let typ_settings = process_header(db, block_declaration);

  let Some(body) = block_declaration.body() else {
    return EvaluatedTyp {
      typ_settings,
      ..EvaluatedTyp::default()
    }
    .into_element_typ(db, name.to_string());
  };

  /* Type fields */
  let fields = process_fields(db, file, &body);
  let typ_field_declarations = fields.field_declarations;
  let typ_rest_attribute = fields.rest_attribute;

  /* In [field] types, [attribute] fields (without [arg]) must be bool or bool? */
  if typ_settings.is_field_typ {
    for (field_name, field) in &typ_field_declarations {
      if field.is_attribute() && !field.is_arg() {
        let is_valid = field
          .typ
          .resolve(db)
          .is_some_and(|typ| is_bool_or_nullable_bool(db, typ));
        if !is_valid {
          let (start, end) = field.span();
          Accumulator::accumulate(
            Diagnostics(
              DiagnosticKind::FieldAttributeMustBeBool {
                field_name: field_name.clone(),
              }
              .into_error(start, end),
            ),
            db,
          );
        }
      }
    }
  }

  /* Type nested types */
  let typ_nested_typs = process_nested_typs(db, file, &body);

  /* Type namespace */
  let typ_namespace = process_namespace(db, file, &body, &typ_field_declarations);

  /* Type method */
  let vtable = process_member_funcs(db, file, &body);

  /* Type constraints */
  let typ_constraints = process_constraints(db, &body);

  EvaluatedTyp {
    vtable,
    typ_settings,
    typ_field_declarations,
    typ_rest_attribute,
    typ_nested_typs,
    typ_namespace,
    typ_constraints,
  }
  .into_element_typ(db, name.to_string())
}

/* Header: `type Name [element, field, use: expand namespaces]` */

/// Evaluates the type settings from the declaration header
fn process_header(
  db: &dyn Database,
  block_declaration: &BlockElementDeclaration,
) -> EvaluatedTypSettings {
  /* Extract type settings */
  let Some(declaration) = ElementDeclaration::cast(block_declaration.syntax().clone()) else {
    return EvaluatedTypSettings::default();
  };
  let Some(settings) = declaration.setting_list() else {
    return EvaluatedTypSettings::default();
  };

  /* Iterate the settings items */
  let mut result = EvaluatedTypSettings::default();

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
      "use" => {
        let value = item.value().map(|value| value.as_str()).unwrap_or_default();
        match value.as_str() {
          "expand namespaces" => result.on_use = OnUsePolicy::ExpandReferences,
          "follow refs" => result.on_use = OnUsePolicy::FollowRefs,
          _ => {
            Diagnostics::accumulate(
              Diagnostics(
                DiagnosticKind::UnrecognizedOnUseValue { value }
                  .into_error(item.syntax().start_offset(), item.syntax().end_offset()),
              ),
              db,
            );
          }
        }
      }
      _ => {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::UnrecognizedTypSetting {
              setting: setting.to_string(),
            }
            .into_error(item.syntax().start_offset(), item.syntax().end_offset()),
          ),
          db,
        );
      }
    }
  }

  /* Post validate the settings */
  if result.is_element_typ && result.is_field_typ {
    Diagnostics::accumulate(
      Diagnostics(DiagnosticKind::ConflictingTypRoles.into_error(
        settings.syntax().start_offset(),
        settings.syntax().end_offset(),
      )),
      db,
    );
  }

  result
}

/* Fields: `name Type [annotations]` lines in the body */

struct EvaluatedTypFields<'db> {
  field_declarations: BTreeMap<String, EvaluatedTypField<'db>>,
  rest_attribute: Option<LazyTyp<'db>>,
}

/// Collects and evaluates all field declarations from the type body
fn process_fields<'db>(
  db: &'db dyn Database,
  file: File,
  body: &BlockElementDeclarationBody,
) -> EvaluatedTypFields<'db> {
  let mut result = EvaluatedTypFields {
    field_declarations: BTreeMap::new(),
    rest_attribute: None,
  };

  /* Extract out all fields and rest attributes */
  let field_declarations = body.fields();

  for field in field_declarations {
    let mut args = field.args();

    /* Process field name */
    let Some(first_arg) = args.next() else {
      continue;
    };

    let Some((field_name, is_rest)) = parse_field_name(db, &field, first_arg) else {
      continue;
    };

    /* Process field type */

    let field_typ = parse_field_typ(db, file, &field_name, args.next());

    /* Handle rest attribute */
    if is_rest {
      // More than 1 rest attribute, raise error
      if result.rest_attribute.is_some() {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::DuplicateRestAttribute
              .into_error(field.syntax().start_offset(), field.syntax().end_offset()),
          ),
          db,
        );
      } else {
        // First rest attribute
        result.rest_attribute = Some(field_typ);
      }
      continue;
    }

    /* Process field settings */

    let settings = parse_field_settings(&field);

    let declaration = EvaluatedTypField {
      typ: field_typ,
      arg_position: settings.arg_position,
      flags: settings.flags,
      setting_rename: settings.setting_rename,
      node: field.syntax().clone(),
    };

    validate_declaration_field_labels(db, &field_name, &declaration); // Validate that a field having `declaration` type must carry `label` or `label alias`
    validate_field_annotation_consistency(db, &field_name, &declaration); // Validate that a field cannot be of multiple roles at the same time (with exceptions)

    result.field_declarations.insert(field_name, declaration);
  }

  /* Process attributes in type */
  disallow_colon_syntax(db, body.attributes()); // Disallow

  /* Post-process: All args must form a contiguous range */
  validate_contiguous_arg_index(
    db,
    result
      .field_declarations
      .iter()
      .map(|(key, value)| (key.as_str(), value)),
  );

  /* Post-process: At most one [label] and one [label alias] */
  validate_unique_label_fields(
    db,
    result
      .field_declarations
      .iter()
      .map(|(key, value)| (key.as_str(), value)),
  );

  /* Post-process: If have alias, must have label */
  validate_alias_has_primary_label(
    db,
    result
      .field_declarations
      .iter()
      .map(|(key, value)| (key.as_str(), value)),
  );
  result
}

/// Parses the first arg as the field name, handling plain idents and rest-attribute `[name]` syntax
fn parse_field_name(
  db: &dyn Database,
  field: &ElementFieldDeclaration,
  first_arg: ElementFieldDeclarationArg,
) -> Option<(String, bool)> {
  if let Some(list) = first_arg
    .expr()
    .and_then(|expr| ListExpr::cast(expr.syntax().clone()))
  {
    // rest attribute: `[name]` - exactly one ident inside brackets
    let mut items = list.items();
    let first_item = items.next().and_then(|item| item.as_name());
    if items.next().is_some() {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::MalformedRestAttribute
            .into_error(field.syntax().start_offset(), field.syntax().end_offset()),
        ),
        db,
      );
      return None;
    }
    first_item.map(|name| (name, true))
  } else {
    first_arg
      .expr()
      .and_then(|expr| expr.as_name())
      .map(|name| (name, false))
  }
}

/// Resolves the second arg as a lazy type, emitting a diagnostic if the expression is invalid
fn parse_field_typ<'db>(
  db: &'db dyn Database,
  file: File,
  field_name: &str,
  type_arg: Option<ElementFieldDeclarationArg>,
) -> LazyTyp<'db> {
  let resolved = type_arg
    .as_ref()
    .and_then(|arg| arg.expr())
    .and_then(|expr| pretype_evaluate_lazy_typ(db, file, &expr));

  if resolved.is_none()
    && let Some(arg) = &type_arg
  {
    Diagnostics::accumulate(
      Diagnostics(
        DiagnosticKind::InvalidFieldTyp {
          field_name: field_name.to_string(),
        }
        .into_error(arg.syntax().start_offset(), arg.syntax().end_offset()),
      ),
      db,
    );
  }

  resolved.unwrap_or_else(|| LazyTyp::eager(ObjTyp::new(db).into()))
}

struct EvaluatedFieldAnnotations {
  arg_position: Option<ArgPosition>,
  flags: FieldAnnotationFlags,
  setting_rename: Option<String>,
}

/// Parses the setting list on a field declaration into annotation flags and arg position
fn parse_field_settings(field: &ElementFieldDeclaration) -> EvaluatedFieldAnnotations {
  let mut result = EvaluatedFieldAnnotations {
    arg_position: None,
    flags: FieldAnnotationFlags::empty(),
    setting_rename: None,
  };

  for item in field
    .children::<SettingList>()
    .flat_map(|setting_list| setting_list.items().collect::<Vec<_>>())
  {
    let Some(item_name) = item.name() else {
      continue;
    };
    match item_name.name().as_str() {
      "attribute" => result.flags |= FieldAnnotationFlags::ATTRIBUTE,
      "label" => result.flags |= FieldAnnotationFlags::LABEL,
      "label alias" => result.flags |= FieldAnnotationFlags::LABEL_ALIAS,
      "unique" => result.flags |= FieldAnnotationFlags::UNIQUE,
      "csv" => result.flags |= FieldAnnotationFlags::CSV,
      "text" => result.flags |= FieldAnnotationFlags::TEXT,
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
        result.flags |= FieldAnnotationFlags::SETTING;
        result.setting_rename = item
          .value()
          .and_then(|value| value.expr())
          .and_then(|expr| expr.as_name());
      }
      _ => {}
    }
  }

  result
}

/// Emits an error for each colon-syntax (`name: Type`) attribute found - invalid inside a type body
fn disallow_colon_syntax(
  db: &dyn Database,
  attributes: impl IntoIterator<Item = ElementAttributeDeclaration>,
) {
  for attribute in attributes {
    let field_name = match attribute
      .name()
      .and_then(|name| name.name())
      .and_then(|expr| expr.as_name())
    {
      Some(name) => name,
      None => continue,
    };
    Diagnostics::accumulate(
      Diagnostics(
        DiagnosticKind::ColonDeclarationInTypBody { field_name }.into_error(
          attribute.syntax().start_offset(),
          attribute.syntax().end_offset(),
        ),
      ),
      db,
    );
  }
}

/* Nested types: `type Name [field|element] { ... }` blocks in the body */

/// Evaluates nested type blocks into field and element nested type maps
fn process_nested_typs<'db>(
  db: &'db dyn Database,
  file: File,
  body: &BlockElementDeclarationBody,
) -> EvaluatedTypNestedTyps<'db> {
  let typ_declarations = body.typ_declarations();

  /* Collect nested types */
  let mut nested_typs = EvaluatedTypNestedTyps::default();

  for nested_typ in typ_declarations {
    let Some(declaration) = ElementDeclaration::cast(nested_typ.syntax().clone()) else {
      continue;
    };
    let Some(declaration_name) = declaration
      .target_name()
      .and_then(|typ| typ.name())
      .map(|fragment| fragment.name().to_string())
    else {
      continue;
    };

    // Check if this nested type has an element or field role
    let has_element = declaration.is_nested_element_typ();
    let has_field = declaration.is_nested_field_typ();

    // Each nested type must have a role
    if !has_element && !has_field {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::NestedTypMissingRole {
            typ_name: declaration_name.clone(),
          }
          .into_error(
            nested_typ.syntax().start_offset(),
            nested_typ.syntax().end_offset(),
          ),
        ),
        db,
      );
      continue;
    }

    let evaluated_nested_typ =
      pretype_evaluate_typ_element(db, file, &declaration_name, &nested_typ);
    if has_element {
      nested_typs
        .element_nested_typs
        .insert(declaration_name, LazyTyp::eager(evaluated_nested_typ));
    } else {
      if !nested_typs.field_nested_typs.is_empty() {
        Diagnostics::accumulate(
          Diagnostics(DiagnosticKind::MultipleNestedFieldTyps.into_error(
            nested_typ.syntax().start_offset(),
            nested_typ.syntax().end_offset(),
          )),
          db,
        );
      } else {
        nested_typs
          .field_nested_typs
          .insert(declaration_name, LazyTyp::eager(evaluated_nested_typ));
      }
    }
  }

  // Disallow any nested elements
  for element in body.disallowed_elements() {
    let element_typ = ElementDeclaration::cast(element.syntax().clone())
      .and_then(|element| element.typ())
      .map(|typ| typ.text().trim().to_string())
      .unwrap_or_else(|| "<unknown>".to_string());

    Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::DisallowedNestedElement { element_typ }.into_error(
          element.syntax().start_offset(),
          element.syntax().end_offset(),
        ),
      ),
      db,
    );
  }

  nested_typs
}

/* Namespace: `namespace { name | get name(): Type { } }` block */

/// Evaluates the namespace block into re-exported field types and getter functions
fn process_namespace<'db>(
  db: &'db dyn Database,
  file: File,
  body: &BlockElementDeclarationBody,
  field_declarations: &BTreeMap<String, EvaluatedTypField<'db>>,
) -> TypNamespace<'db> {
  let Some(namespace_body) = body.namespace_body() else {
    return TypNamespace::default();
  };

  let mut members = BTreeMap::new();
  let mut getters = BTreeMap::new();
  let mut expanded_field: Option<String> = None;
  let mut expanded_item_typ: Option<LazyTyp<'db>> = None;

  // Field re-exports: `namespace { name }` or `namespace { name [expand] }`
  for field in namespace_body.fields() {
    let Some(name_arg) = field.args().next() else {
      continue;
    };
    let Some(field_name) = name_arg.expr().and_then(|expr| expr.as_name()) else {
      continue;
    };

    let is_expand = field
      .children::<SettingList>()
      .any(|sl| sl.has_item("expand"));

    if !is_expand {
      // Plain re-export: exposes the field's type directly
      if let Some(declaration) = field_declarations.get(&field_name) {
        members.insert(field_name, declaration.typ);
      } else {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::UnknownNamespaceExport { field_name }
              .into_error(field.syntax().start_offset(), field.syntax().end_offset()),
          ),
          db,
        );
      }
    } else {
      // Expanded re-export: `name [expand]` exposes list items by their `name` field
      if expanded_field.is_some() {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::DuplicateExpandedNamespaceField {
              field_name: field_name.clone(),
            }
            .into_error(field.syntax().start_offset(), field.syntax().end_offset()),
          ),
          db,
        );
        continue;
      }
      let Some(declaration) = field_declarations.get(&field_name) else {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::UnknownNamespaceExport {
              field_name: field_name.clone(),
            }
            .into_error(field.syntax().start_offset(), field.syntax().end_offset()),
          ),
          db,
        );
        continue;
      };
      // Validate the field is a list type and extract the item type
      let item_lazy = declaration.typ.resolve(db).and_then(|resolved| {
        if let Typ::List(list_typ) = resolved {
          *list_typ.element(db)
        } else {
          None
        }
      });
      if let Some(item) = item_lazy {
        expanded_field = Some(field_name);
        expanded_item_typ = Some(item);
      } else {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::ExpandedNamespaceFieldMustBeList {
              field_name: field_name.clone(),
            }
            .into_error(field.syntax().start_offset(), field.syntax().end_offset()),
          ),
          db,
        );
      }
    }
  }

  // Colon-syntax in namespace block is not a valid form
  for attribute in namespace_body.attributes() {
    let field_name = match attribute
      .name()
      .and_then(|name| name.name())
      .and_then(|expr| expr.as_name())
    {
      Some(name) => name,
      None => continue,
    };
    Diagnostics::accumulate(
      Diagnostics(
        DiagnosticKind::ColonDeclarationInNamespaceBlock { field_name }.into_error(
          attribute.syntax().start_offset(),
          attribute.syntax().end_offset(),
        ),
      ),
      db,
    );
  }

  // Getters: `get name(): Type { }`, computed property exposed via `.name`
  for getter in namespace_body.get_declarations() {
    let Some(name_node) = getter.name() else {
      continue;
    };
    let getter_name = name_node.text().trim().to_string();
    let typ = getter
      .return_typ()
      .and_then(|return_typ| return_typ.typ_expr())
      .and_then(|expr| pretype_evaluate_lazy_typ(db, file, &expr))
      .unwrap_or_else(|| LazyTyp::eager(ObjTyp::new(db).into()));
    members.insert(getter_name.clone(), typ);

    let signature = FuncSignature::new(db, vec![], typ);
    let func_obj = FuncObj::new(
      db,
      getter_name.clone(),
      FuncTyp::new(db, signature),
      Some(getter.syntax().clone()),
      None,
    );
    getters.insert(getter_name, func_obj);
  }

  TypNamespace {
    members,
    getters,
    expanded_field,
    expanded_item_typ,
  }
}

/* Member functions: `fn name(params): ReturnType { ... }` and `get name(): Type { }` */

/// Evaluates member function declarations into a name-to-FuncObj vtable
fn process_member_funcs<'db>(
  db: &'db dyn Database,
  file: File,
  body: &BlockElementDeclarationBody,
) -> BTreeMap<String, FuncObj<'db>> {
  let mut vtable = BTreeMap::new();
  for func_declaration in body.func_declarations() {
    let Some(name_node) = func_declaration.name() else {
      continue;
    };
    let func_name = name_node.text().trim().to_string();
    // params and ret are lazy - symbol references resolved later, not at pretype
    let return_typ = func_declaration
      .return_typ()
      .and_then(|return_typ| return_typ.typ_expr())
      .and_then(|expr| pretype_evaluate_lazy_typ(db, file, &expr))
      .unwrap_or_else(|| LazyTyp::eager(ObjTyp::new(db).into()));

    let mut param_typs = Vec::new();
    if let Some(params_node) = func_declaration.params() {
      for param in params_node.params() {
        let param_typ = param
          .typ_expr()
          .and_then(|expr| pretype_evaluate_lazy_typ(db, file, &expr))
          .unwrap_or_else(|| LazyTyp::eager(ObjTyp::new(db).into()));
        param_typs.push(param_typ);
      }
    }

    let signature = FuncSignature::new(db, param_typs, return_typ);
    let func_obj = FuncObj::new(
      db,
      func_name.clone(),
      FuncTyp::new(db, signature),
      Some(func_declaration.syntax().clone()),
      None,
    );
    vtable.insert(func_name, func_obj);
  }
  vtable
}

/// Parses `n` or `n..` from a setting value expression into an (index, is_variadic) pair
fn parse_positional_index(expr: &Expr) -> Option<(usize, bool)> {
  if let Some(postfix) = PostfixExpr::cast(expr.syntax().clone())
    && postfix.has_op("..")
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

/* Constraints: `constraints { name: closure }` block */

/// Evaluates the constraints block into a name-to-TypConstraint map
fn process_constraints(
  db: &dyn Database,
  body: &BlockElementDeclarationBody,
) -> BTreeMap<String, TypConstraint> {
  let mut constraints = BTreeMap::new();
  if let Some(block) = body.constraints()
    && let Some(body) = block.body()
  {
    for attribute in body.attributes() {
      if let Some(entry) = extract_constraint_entry(&attribute) {
        let (_, constraint_definition) = validate_constraint_entry(db, &entry);
        if let Some(definition) = constraint_definition {
          constraints.insert(definition.name.clone(), definition);
        }
      }
    }
  }
  constraints
}

/// Returns true if the type is `bool` or `bool | null`
fn is_bool_or_nullable_bool(db: &dyn Database, typ: Typ) -> bool {
  match typ {
    Typ::Bool(_) => true,
    Typ::Sum(sum) => {
      let members = sum.members(db);
      members.len() == 2
        && members.iter().any(|member| matches!(member, Typ::Bool(_)))
        && members.iter().any(|member| matches!(member, Typ::Null(_)))
    }
    _ => false,
  }
}
