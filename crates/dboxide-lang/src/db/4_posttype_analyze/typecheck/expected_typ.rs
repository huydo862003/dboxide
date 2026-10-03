use salsa::{Database, tracked};

use super::{collect_operator_overloads, extract_func_typ};
use crate::ast::{
  AstNode, CallExpr, ClosureExpr, ElementAttributeDeclaration, ElementDeclaration,
  ElementFieldDeclaration, FuncDeclaration, IndexExpr, InfixExpr, PrefixExpr, RedNode,
  SettingListItem, SyntaxKind, TernaryExpr,
};
use crate::db::posttype_analyze::posttype_resolve_referee;
use crate::db::pretype_analyze::get_declaration_typ;
use crate::db::types::{BoolTyp, ElementTyp, File, IntegerTyp, NeverTyp, NullTyp, SumTyp, Typ};

/// Returns the expected Typ at a given node position, if determinable from context
#[tracked]
pub fn get_node_expected_typ<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
) -> Option<Typ<'db>> {
  let mut cursor = node;
  loop {
    let parent = cursor.parent()?;
    match parent.kind() {
      /* Type-annotation boundaries */
      SyntaxKind::ElementDeclarationTyp
      | SyntaxKind::FuncDeclarationReturnTyp
      | SyntaxKind::UseSpecifierName
      | SyntaxKind::UseSpecifierAlias => return None,

      /* Field line arg in element body */
      SyntaxKind::ElementFieldDeclarationArg => {
        return get_expected_field_line_arg_typ(db, file, &parent);
      }

      /* Setting value */
      SyntaxKind::SettingListItemValue => {
        return get_expected_setting_typ(db, file, &parent);
      }

      /* Attribute value */
      SyntaxKind::ElementAttributeDeclarationValue => {
        return get_expected_attribute_typ(db, file, &parent);
      }

      /* Infix expression */
      SyntaxKind::InfixExpr => {
        if let Some(infix) = InfixExpr::cast(parent.clone()) {
          match infix.op().as_deref() {
            Some(".") | Some("@") => {}
            Some(op) => {
              // `??` coalesce: LHS expects T|null, RHS expects T
              if op == "??" {
                if let Some(expected) = *get_node_expected_typ(db, file, parent.clone()) {
                  if infix
                    .left()
                    .is_some_and(|lhs| lhs.syntax().start_offset() == cursor.start_offset())
                  {
                    return Some(SumTyp::create(
                      db,
                      vec![expected, Typ::Null(NullTyp::new(db))],
                    ));
                  }
                  return Some(expected);
                }
                return None;
              }

              let is_lhs = infix
                .left()
                .is_some_and(|lhs| lhs.syntax().start_offset() == cursor.start_offset());
              let overloads = collect_operator_overloads(db, file, &cursor, op);
              let param_index = if is_lhs { 0 } else { 1 };
              let mut typs: Vec<Typ> = Vec::new();
              for func_typ in &overloads {
                if let Some(typ) = func_typ
                  .params(db)
                  .get(param_index)
                  .and_then(|param| param.resolve(db))
                  && !typs.contains(&typ)
                {
                  typs.push(typ);
                }
              }
              if !typs.is_empty() {
                return Some(SumTyp::create(db, typs));
              }
            }
            _ => {}
          }
        }
        cursor = parent;
      }

      /* Prefix expression */
      SyntaxKind::PrefixExpr => {
        if PrefixExpr::cast(parent.clone())
          .and_then(|parent| parent.op())
          .as_deref()
          == Some("!")
        {
          return Some(Typ::Bool(BoolTyp::new(db)));
        }
        cursor = parent;
      }

      /* Ternary expression */
      SyntaxKind::TernaryExpr => {
        if let Some(ternary) = TernaryExpr::cast(parent.clone())
          && ternary
            .condition()
            .is_some_and(|condition| condition.syntax().start_offset() == cursor.start_offset())
        {
          return Some(Typ::Bool(BoolTyp::new(db)));
        }
        return *get_node_expected_typ(db, file, parent.clone());
      }

      /* Call expression */
      SyntaxKind::CallExpr => {
        if let Some(call) = CallExpr::cast(parent.clone())
          && let Some(callee) = call.callee()
        {
          // Callee position: no constraint, keep walking up
          if callee.syntax().start_offset() == cursor.start_offset() {
            cursor = parent;
            continue;
          }

          // Arg position: derive param type from callee's signature
          // Out-of-bounds args get Never (no valid type can satisfy an extra arg)
          if let Some(arg_index) = call
            .args()
            .position(|arg| arg.syntax().start_offset() == cursor.start_offset())
            && let Some(result) = *posttype_resolve_referee(db, file, callee.syntax().clone())
            && let Some(symbol) = result.symbol
            && let Some(func_typ) = extract_func_typ(db, symbol)
          {
            let params = func_typ.params(db);
            return if arg_index < params.len() {
              params[arg_index].resolve(db)
            } else {
              Some(Typ::Never(NeverTyp::new(db)))
            };
          }
        }
        cursor = parent;
      }

      /* Index expression */
      SyntaxKind::IndexExpr => {
        if let Some(index_expr) = IndexExpr::cast(parent.clone())
          && let Some(base) = index_expr.base()
        {
          if base.syntax().start_offset() == cursor.start_offset() {
            cursor = parent;
            continue;
          }

          // Subscript position: Integer for lists, key type for maps
          if let Some(base_typ) =
            *super::actual_typ::get_node_actual_typ(db, file, base.syntax().clone())
          {
            return match base_typ {
              Typ::List(_) => Some(Typ::Integer(IntegerTyp::new(db))),
              Typ::Map(map_typ) => map_typ.key(db).and_then(|key| key.resolve(db)),
              _ => Some(Typ::Integer(IntegerTyp::new(db))),
            };
          }
        }
        cursor = parent;
      }

      /* List literal */
      SyntaxKind::ListExpr => {
        if let Some(Typ::List(list_typ)) = *get_node_expected_typ(db, file, parent.clone()) {
          return list_typ.element(db).and_then(|element| element.resolve(db));
        }
        return None;
      }

      /* Closure body: constraint closures expect Bool */
      SyntaxKind::ClosureExpr => {
        if let Some(closure) = ClosureExpr::cast(parent.clone())
          && closure
            .body()
            .is_some_and(|body| body.syntax().start_offset() == cursor.start_offset())
        {
          return Some(Typ::Bool(BoolTyp::new(db)));
        }
        cursor = parent;
      }

      /* Function body: last expression expects the declared return type */
      SyntaxKind::BlockElementDeclarationBody => {
        if let Some(func_declaration) = parent.parent().and_then(FuncDeclaration::cast)
          && let Some(ret_typ) = func_declaration
            .return_typ()
            .and_then(|ret_typ| *get_declaration_typ(db, file, ret_typ.syntax().clone()))
        {
          return Some(ret_typ);
        }
        cursor = parent;
      }

      _ => cursor = parent,
    }
  }
}

/// Get expected type for a field line arg position in an element body
fn get_expected_field_line_arg_typ<'db>(
  db: &'db dyn Database,
  file: File,
  arg_node: &RedNode,
) -> Option<Typ<'db>> {
  let field_declaration_node = arg_node.parent()?;
  let body_node = field_declaration_node.parent()?;
  let element_node = body_node.parent()?;

  // In a type body, field args are type annotations, not values
  if ElementDeclaration::cast(element_node.clone())?.is_typ_declaration() {
    return None;
  }

  let field_declaration = ElementFieldDeclaration::cast(field_declaration_node.clone())?;
  let arg_index = field_declaration
    .args()
    .position(|arg| arg.syntax().start_offset() == arg_node.start_offset())?;

  let Some(Typ::Element(element_typ)) = *get_declaration_typ(db, file, element_node) else {
    return None;
  };

  resolve_field_line_arg_typ(db, element_typ, &field_declaration, arg_index)
}

/// Look up expected type for a specific arg position in a field line
fn resolve_field_line_arg_typ<'db>(
  db: &'db dyn Database,
  element_typ: ElementTyp<'db>,
  field_declaration: &ElementFieldDeclaration,
  arg_index: usize,
) -> Option<Typ<'db>> {
  let field_nested_typs = &element_typ.typ_nested_typs(db).field_nested_typs;
  if field_nested_typs.is_empty() {
    return None;
  }

  let (field_element_typ, arg_offset) = if field_nested_typs.len() == 1 {
    let (_, lazy) = field_nested_typs.iter().next()?;
    let Typ::Element(element_typ) = lazy.resolve(db)? else {
      return None;
    };
    (element_typ, 0usize)
  } else {
    // Multiple field types: arg[0] is the type discriminator
    if arg_index == 0 {
      return None;
    }
    let discriminator = field_declaration.args().next()?.expr()?.as_name()?;
    let (_, lazy) = field_nested_typs
      .iter()
      .find(|(key, _)| key.eq_ignore_ascii_case(&discriminator))?;
    let Typ::Element(element_typ) = lazy.resolve(db)? else {
      return None;
    };
    (element_typ, 1usize)
  };

  if arg_index < arg_offset {
    return None;
  }
  field_element_typ
    .typ_element_arg_fields(db)
    .get(&(arg_index - arg_offset))
    .and_then(|arg_field| arg_field.typ.resolve(db))
}

/// Get expected type for a setting value position
fn get_expected_setting_typ<'db>(
  db: &'db dyn Database,
  file: File,
  value_node: &RedNode,
) -> Option<Typ<'db>> {
  let item_node = value_node.parent()?;
  let setting_list_node = item_node.parent()?;
  let item = SettingListItem::cast(item_node.clone())?;
  let setting_name = item.name()?.name();

  let container_node = setting_list_node.parent()?;

  let element_node = match container_node.kind() {
    SyntaxKind::ElementFieldDeclaration => container_node.parent()?.parent()?,
    _ => container_node,
  };

  let Some(Typ::Element(element_typ)) = *get_declaration_typ(db, file, element_node) else {
    return None;
  };

  element_typ
    .typ_element_settings(db)
    .get(&setting_name)
    .and_then(|lazy| lazy.resolve(db))
}

/// Get expected type for an attribute value position
fn get_expected_attribute_typ<'db>(
  db: &'db dyn Database,
  file: File,
  attribute_value_node: &RedNode,
) -> Option<Typ<'db>> {
  let attribute_declaration_node = attribute_value_node.parent()?;
  let attribute_declaration =
    ElementAttributeDeclaration::cast(attribute_declaration_node.clone())?;
  let attribute_name = attribute_declaration.name()?.name()?.as_name()?;

  let body_node = attribute_declaration_node.parent()?;
  let element_node = body_node.parent()?;

  let Some(Typ::Element(element_typ)) = *get_declaration_typ(db, file, element_node) else {
    return None;
  };

  element_typ
    .attribute_definitions(db)
    .into_iter()
    .find(|attribute| attribute.name == attribute_name)?
    .typ
    .resolve(db)
}
