use salsa::Database;

use crate::OStringObj;
use crate::ast::{
  AstNode, CallExpr, ColorExpr, CsvExpr, DqStringExpr, Expr, IdentExpr, IndexExpr, InfixExpr,
  ListExpr, NumberExpr, OqStringExpr, ParenExpr, PostfixExpr, SqStringExpr, TqStringExpr,
  TupleExpr,
};
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::{SymbolKind, VirtualFunctionKind, VirtualModuleKind};
use crate::db::types::{
  BoolObj, ColorObj, File, FloatObj, IntegerObj, ListObj, ListTyp, NullObj, NullTyp, Obj, ObjTyp,
  RuntimeObj, StaticTyp, StringObj, SumTyp, Typ,
};

use super::pretype_evaluate_typ_symbol;
use crate::db::pretype_analyze::pretype_resolve_referee;

/// Evaluates an expression in pretype context: resolves identifiers to types/schemas only
pub fn pretype_evaluate_node<'db>(
  db: &'db dyn Database,
  file: File,
  expr: &Expr,
) -> Option<Obj<'db>> {
  let node = expr.syntax().clone();

  /* Ident */
  if let Some(ident) = IdentExpr::cast(node.clone())
    && !matches!(ident.name().as_str(), "true" | "false" | "null")
  {
    let symbol = (*pretype_resolve_referee(db, file, node))?;
    return *pretype_evaluate_typ_symbol(db, symbol);
  }

  /* Structural */
  if let Some(paren) = ParenExpr::cast(node.clone()) {
    // `(expr)`: transparent grouping, evaluates inner expression
    return paren
      .inner()
      .and_then(|inner| pretype_evaluate_node(db, file, &inner));
  }

  if let Some(index) = IndexExpr::cast(node.clone()) {
    let base_expr = index.base()?;
    let base = pretype_evaluate_node(db, file, &base_expr)?;
    if let Some(base_typ) = base.as_typ() {
      if index.indices().next().is_none() {
        // `T[]`: no indices, shorthand for a list type
        return Some(Obj::Typ(
          ListTyp::new(db, Some(LazyTyp::eager(base_typ))).into(),
        ));
      }
      // `T[A, B]`: type instantiation, applies type arguments to a generic type
      let lazy_args: Vec<LazyTyp<'db>> = index
        .indices()
        .filter_map(|arg| pretype_evaluate_lazy_typ(db, file, &arg))
        .collect();
      if let Some(instantiated) = base_typ.instantiate(db, lazy_args) {
        return Some(Obj::Typ(instantiated));
      }
    }
    // `value[key]`: runtime index access on a non-type value
    let args: Vec<Obj<'db>> = index
      .indices()
      .filter_map(|arg| pretype_evaluate_node(db, file, &arg))
      .collect();
    if args.len() == 1 {
      return base.index(db, &args[0]);
    }
    return None;
  }

  /* Type operators */
  if let Some(postfix) = PostfixExpr::cast(node.clone())
    && postfix.has_op("?")
  {
    // `?`: makes a type nullable - `T?` = `T | null`
    let inner = pretype_evaluate_node(db, file, &postfix.operand()?)?.as_typ()?;
    let null: Typ = NullTyp::new(db).into();
    return Some(Obj::Typ(SumTyp::create(db, vec![inner, null])));
  }

  if let Some(infix) = InfixExpr::cast(node.clone())
    && infix.has_op(".")
  {
    // `.`: namespace traversal - looks up a member in the left type's namespace
    let left = pretype_evaluate_node(db, file, &infix.left()?)?.as_typ()?;
    let member_name = infix.right()?.as_name()?;
    let lazy = left.get_namespace_member(db, &member_name)?;
    let resolved = lazy.resolve(db)?;
    return Some(Obj::Typ(resolved));
  }

  if let Some(infix) = InfixExpr::cast(node.clone())
    && infix.has_op("|")
  {
    // `|`: type union - `A | B` builds a sum type
    let left = pretype_evaluate_node(db, file, &infix.left()?)
      .and_then(|obj| obj.coerce_to_union_member_typ(db))?;
    let right = pretype_evaluate_node(db, file, &infix.right()?)
      .and_then(|obj| obj.coerce_to_union_member_typ(db))?;
    return Some(Obj::Typ(SumTyp::create(db, vec![left, right])));
  }

  /* Literals */
  if let Some(ident) = IdentExpr::cast(node.clone()) {
    // `true`, `false`, `null` parse as ident tokens
    return match ident.name().as_str() {
      "true" => Some(BoolObj::new(db, true).into()),
      "false" => Some(BoolObj::new(db, false).into()),
      "null" => Some(NullObj::new(db).into()),
      _ => None,
    };
  }

  if let Some(single_quoted_string) = SqStringExpr::cast(node.clone()) {
    // single-quoted string: `'text'`
    return Some(StringObj::new(db, single_quoted_string.content()).into());
  }

  if let Some(opaque_string) = OqStringExpr::cast(node.clone()) {
    // opaque string: `` `text` `` - preserved as-is, no escaping
    return Some(OStringObj::new(db, opaque_string.content()).into());
  }

  if let Some(double_quoted_string) = DqStringExpr::cast(node.clone()) {
    // double-quoted strings in type position are referees (e.g. `"My Table"` as a symbol name with spaces)
    if let Some(symbol) = *pretype_resolve_referee(db, file, node) {
      return *pretype_evaluate_typ_symbol(db, symbol);
    }
    // outside type position: treat as a string literal
    return Some(StringObj::new(db, double_quoted_string.content()).into());
  }

  if let Some(triple_quoted_string) = TqStringExpr::cast(node.clone()) {
    // triple-quoted multiline string: `'''text'''` (dedented and unescaped)
    return Some(StringObj::new(db, triple_quoted_string.value()).into());
  }

  if let Some(number) = NumberExpr::cast(node.clone()) {
    // integer or float literal
    if let Some(integer) = number.int_value() {
      return Some(IntegerObj::new(db, integer).into());
    }
    return Some(FloatObj::new(db, number.raw_text()).into());
  }

  if let Some(color) = ColorExpr::cast(node.clone()) {
    // `#rgb` / `#rrggbb`: hex color literal
    let (r, g, b) = color.value()?;
    return Some(ColorObj::new(db, r, g, b).into());
  }

  if let Some(list) = ListExpr::cast(node.clone()) {
    // `[a, b, c]`: bracket list literal
    let items: Vec<Obj<'db>> = list
      .items()
      .filter_map(|item| pretype_evaluate_node(db, file, &item))
      .collect();
    return Some(ListObj::new(db, items).into());
  }

  if let Some(tuple) = TupleExpr::cast(node.clone()) {
    // `(a, b, c)` / `()`: tuple literal, evaluates as a list
    let items: Vec<Obj<'db>> = tuple
      .items()
      .filter_map(|item| pretype_evaluate_node(db, file, &item))
      .collect();
    return Some(ListObj::new(db, items).into());
  }

  if let Some(csv) = CsvExpr::cast(node.clone()) {
    // `a, b, c`: comma-separated list literal
    let items: Vec<Obj<'db>> = csv
      .items()
      .filter_map(|item| pretype_evaluate_node(db, file, &item))
      .collect();
    return Some(ListObj::new(db, items).into());
  }

  /* Calls: supported only when the callee resolves to a metatype virtual function */
  if let Some(call) = CallExpr::cast(node) {
    let callee_node = call.callee()?.syntax().clone();
    let symbol = (*pretype_resolve_referee(db, file, callee_node))?;
    if let SymbolKind::VirtualFunction(VirtualModuleKind::Metatype, kind) =
      symbol.original_symbol(db).kind(db)
    {
      return evaluate_virtual_call(db, file, &call, kind);
    }
    return None;
  }

  None
}

/// Evaluates a call to a metatype virtual function
fn evaluate_virtual_call<'db>(
  db: &'db dyn Database,
  file: File,
  call: &CallExpr,
  kind: &VirtualFunctionKind,
) -> Option<Obj<'db>> {
  match kind {
    VirtualFunctionKind::Interpret => {
      let arg_typ = call
        .args()
        .next()
        .and_then(|arg| pretype_evaluate_node(db, file, &arg))
        .and_then(|obj| obj.as_typ());
      Some(Obj::Typ(arg_typ.unwrap_or_else(|| ObjTyp::new(db).into())))
    }
    VirtualFunctionKind::Typeof => {
      let typ = call
        .args()
        .next()
        .and_then(|arg| pretype_evaluate_node(db, file, &arg))
        .map(|arg| arg.get_typ(db))
        .unwrap_or_else(|| ObjTyp::new(db).into());
      Some(Obj::Typ(typ))
    }
  }
}

/// Returns a `LazyTyp` for an expression in pretype context
pub fn pretype_evaluate_lazy_typ<'db>(
  db: &'db dyn Database,
  file: File,
  expr: &Expr,
) -> Option<LazyTyp<'db>> {
  let node = expr.syntax().clone();
  if IdentExpr::cast(node.clone()).is_some()
    && let Some(symbol) = *pretype_resolve_referee(db, file, node)
  {
    return Some(LazyTyp::lazy(symbol));
  }
  pretype_evaluate_node(db, file, expr)
    .and_then(|obj| obj.as_typ())
    .map(LazyTyp::eager)
}
