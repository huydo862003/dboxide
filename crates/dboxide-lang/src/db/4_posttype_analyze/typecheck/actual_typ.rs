use salsa::{Database, tracked};

use super::{collect_operator_overloads, extract_func_typ, resolve_symbol_typ};
use crate::ast::{
  AstNode, CallExpr, DqStringExpr, IdentExpr, IndexExpr, InfixExpr, ListExpr, NumberExpr,
  OqStringExpr, ParenExpr, PrefixExpr, RedNode, SqStringExpr, TernaryExpr, TqStringExpr,
};
use crate::db::posttype_analyze::posttype_resolve_referee;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{
  BoolTyp, File, FloatTyp, IntegerTyp, ListTyp, OStringTyp, StringTyp, SumTyp, Typ,
};
use crate::typecheck::is_subtyp_of;

/// Returns the actual Typ this node produces, bottom-up from resolution
#[tracked]
pub fn get_node_actual_typ<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
) -> Option<Typ<'db>> {
  /* Identifier */
  if IdentExpr::cast(node.clone()).is_some() {
    let result = (*posttype_resolve_referee(db, file, node))?;
    if let Some(symbol) = result.symbol {
      return resolve_symbol_typ(db, symbol);
    }
    return result.resolved_typ;
  }

  /* Number literal */
  if let Some(num) = NumberExpr::cast(node.clone()) {
    let text = num.text();
    let is_float = text.contains('.') || text.contains('e') || text.contains('E');
    return if is_float {
      Some(Typ::Float(FloatTyp::new(db)))
    } else {
      Some(Typ::Integer(IntegerTyp::new(db)))
    };
  }

  /* String literal */
  if DqStringExpr::cast(node.clone()).is_some()
    || SqStringExpr::cast(node.clone()).is_some()
    || TqStringExpr::cast(node.clone()).is_some()
  {
    return Some(Typ::String(StringTyp::new(db)));
  }
  if OqStringExpr::cast(node.clone()).is_some() {
    return Some(Typ::OString(OStringTyp::new(db)));
  }

  /* Parenthesized expression */
  if let Some(paren) = ParenExpr::cast(node.clone()) {
    return paren
      .inner()
      .and_then(|element| *get_node_actual_typ(db, file, element.syntax().clone()));
  }

  /* Infix expression */
  if let Some(infix) = InfixExpr::cast(node.clone()) {
    return resolve_infix_typ(db, file, &infix);
  }

  /* Prefix expression */
  if let Some(prefix) = PrefixExpr::cast(node.clone()) {
    return match prefix.op().as_deref() {
      Some("!") => Some(Typ::Bool(BoolTyp::new(db))),
      Some("-") => prefix
        .operand()
        .and_then(|element| *get_node_actual_typ(db, file, element.syntax().clone())),
      _ => None,
    };
  }

  /* Ternary expression: union of branch types */
  if let Some(ternary) = TernaryExpr::cast(node.clone()) {
    let types: Vec<Typ> = [ternary.then_expr(), ternary.else_expr()]
      .into_iter()
      .flatten()
      .filter_map(|element| *get_node_actual_typ(db, file, element.syntax().clone()))
      .collect();
    return if types.is_empty() {
      None
    } else {
      Some(SumTyp::create(db, types))
    };
  }

  /* Call expression: callee's return type */
  if let Some(call) = CallExpr::cast(node.clone()) {
    if let Some(callee) = call.callee() {
      let result = (*posttype_resolve_referee(db, file, callee.syntax().clone()))?;
      let func_typ = extract_func_typ(db, result.symbol?)?;
      return func_typ.ret(db).resolve(db);
    }
    return None;
  }

  /* Index expression: element type of base */
  if let Some(index_expr) = IndexExpr::cast(node.clone()) {
    if let Some(base) = index_expr.base() {
      let base_typ = (*get_node_actual_typ(db, file, base.syntax().clone()))?;
      return match base_typ {
        Typ::List(list_typ) => list_typ.element(db).and_then(|element| element.resolve(db)),
        Typ::Map(map_typ) => map_typ.value(db).and_then(|value| value.resolve(db)),
        _ => None,
      };
    }
    return None;
  }

  /* List literal: ListTyp with union of element types */
  if let Some(list_expr) = ListExpr::cast(node.clone()) {
    let element_typs: Vec<Typ> = list_expr
      .items()
      .filter_map(|item| *get_node_actual_typ(db, file, item.syntax().clone()))
      .collect();
    let element_lazy = if element_typs.is_empty() {
      None
    } else {
      Some(LazyTyp::eager(SumTyp::create(db, element_typs)))
    };
    return Some(Typ::List(ListTyp::new(db, element_lazy)));
  }

  None
}

/// Derive actual type for an infix expression based on operator
fn resolve_infix_typ<'db>(
  db: &'db dyn Database,
  file: File,
  infix: &InfixExpr,
) -> Option<Typ<'db>> {
  match infix.op().as_deref() {
    // Namespace/field traversal: type from resolved symbol or type-level fallback
    Some(".") | Some("@") => {
      let rhs = infix.right()?;
      let result = (*posttype_resolve_referee(db, file, rhs.syntax().clone()))?;
      if let Some(symbol) = result.symbol {
        resolve_symbol_typ(db, symbol)
      } else {
        result.resolved_typ
      }
    }
    // Operators: narrow overloads by actual operand types, return matching return types
    Some(op) => {
      // `??` coalesce: return type is LHS with null stripped (more precise than the overload)
      if op == "??" {
        let lhs = infix.left()?;
        let lhs_typ = (*get_node_actual_typ(db, file, lhs.syntax().clone()))?;
        return Some(strip_null(db, lhs_typ));
      }
      let overloads = collect_operator_overloads(db, file, infix.syntax(), op);
      if overloads.is_empty() {
        return None;
      }

      // Narrow by actual operand types when available
      let lhs_typ = infix
        .left()
        .and_then(|element| *get_node_actual_typ(db, file, element.syntax().clone()));
      let rhs_typ = infix
        .right()
        .and_then(|element| *get_node_actual_typ(db, file, element.syntax().clone()));

      let matched: Vec<_> = overloads
        .iter()
        .filter(|func_typ| {
          let params = func_typ.params(db);
          let lhs_ok = match (&lhs_typ, params.first()) {
            (Some(actual), Some(param)) => param
              .resolve(db)
              .is_none_or(|param| is_subtyp_of(db, actual, &param)),
            _ => true,
          };
          let rhs_ok = match (&rhs_typ, params.get(1)) {
            (Some(actual), Some(param)) => param
              .resolve(db)
              .is_none_or(|param| is_subtyp_of(db, actual, &param)),
            _ => true,
          };
          lhs_ok && rhs_ok
        })
        .copied()
        .collect();

      let candidates = if matched.is_empty() {
        &overloads
      } else {
        &matched
      };
      let mut typs: Vec<Typ> = Vec::new();
      for func_typ in candidates {
        if let Some(typ) = func_typ.ret(db).resolve(db)
          && !typs.contains(&typ)
        {
          typs.push(typ);
        }
      }
      if typs.is_empty() {
        None
      } else {
        Some(SumTyp::create(db, typs))
      }
    }
    _ => None,
  }
}

/// Strip Null from a nullable type
fn strip_null<'db>(db: &'db dyn Database, typ: Typ<'db>) -> Typ<'db> {
  match typ {
    Typ::Sum(sum) => {
      let non_null: Vec<Typ> = sum
        .members(db)
        .iter()
        .filter(|member| !matches!(member, Typ::Null(_)))
        .copied()
        .collect();
      if non_null.is_empty() {
        typ
      } else {
        SumTyp::create(db, non_null)
      }
    }
    _ => typ,
  }
}
