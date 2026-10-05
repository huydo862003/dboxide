use salsa::Database;

use crate::OStringObj;
use crate::ast::{
  AstNode, DqStringExpr, Expr, IdentExpr, IndexExpr, InfixExpr, NumberExpr, OqStringExpr,
  ParenExpr, PostfixExpr, SqStringExpr,
};
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{BoolObj, FloatObj, IntegerObj, NullTyp, Obj, StringObj, SumTyp, Typ};
use crate::db::types::{RuntimeObj, StaticTyp};

use super::pretype_evaluate_typ_symbol::pretype_evaluate_typ_symbol;
use crate::db::pretype::pretype_resolve_name;
use crate::db::types::File;

pub fn pretype_evaluate_node<'db>(
  db: &'db dyn Database,
  file: File,
  expr: &Expr,
) -> Option<Obj<'db>> {
  let node = expr.syntax().clone();

  if let Some(ident) = IdentExpr::cast(node.clone()) {
    let name = ident.name();
    match name.as_str() {
      "true" => return Some(BoolObj::new(db, true).into()),
      "false" => return Some(BoolObj::new(db, false).into()),
      _ => {}
    }
    let symbol = (*pretype_resolve_name(db, file, node, name))?;
    return *pretype_evaluate_typ_symbol(db, symbol);
  }

  if let Some(index) = IndexExpr::cast(node.clone()) {
    let base_expr = index.base()?;
    let base = pretype_evaluate_node(db, file, &base_expr)?;

    // Type instantiation (List[string], declaration[T], Map[K, V])
    if let Some(base_typ) = base.as_typ() {
      let lazy_args: Vec<LazyTyp<'db>> = index
        .indices()
        .filter_map(|arg| pretype_evaluate_lazy_typ(db, file, &arg))
        .collect();
      if let Some(instantiated) = base_typ.instantiate(db, lazy_args) {
        return Some(Obj::Typ(instantiated));
      }
    }

    // Delegates to RuntimeObj.index (runtime indexing)
    let args: Vec<Obj<'db>> = index
      .indices()
      .filter_map(|arg| pretype_evaluate_node(db, file, &arg))
      .collect();

    if args.len() == 1 {
      return base.index(db, &args[0]);
    }

    return None;
  }

  if let Some(postfix) = PostfixExpr::cast(node.clone())
    && postfix.op().as_deref() == Some("?")
  {
    let inner = pretype_evaluate_node(db, file, &postfix.operand()?)?.as_typ()?;
    let null: Typ = NullTyp::new(db).into();
    return Some(Obj::Typ(SumTyp::create(db, vec![inner, null])));
  }

  if let Some(infix) = InfixExpr::cast(node.clone())
    && infix.op().as_deref() == Some("|")
  {
    let left = pretype_evaluate_node(db, file, &infix.left()?)?.as_typ()?;
    let right = pretype_evaluate_node(db, file, &infix.right()?)?.as_typ()?;
    return Some(Obj::Typ(SumTyp::create(db, vec![left, right])));
  }

  if let Some(single_quoted_string) = SqStringExpr::cast(node.clone()) {
    return Some(StringObj::new(db, single_quoted_string.content()).into());
  }

  if let Some(opaque_string) = OqStringExpr::cast(node.clone()) {
    return Some(OStringObj::new(db, opaque_string.content()).into());
  }

  // Standalone double quoted strings (or parenthesized) are not handled here because they are ambiguous
  // Other subexpressions are always treated as substring
  if let Some(double_quoted_string) = DqStringExpr::cast(node.clone()) {
    if node.parent().is_some_and(|parent| {
      Expr::cast(parent.clone()).is_some() && ParenExpr::cast(parent).is_none()
    }) {
      return Some(StringObj::new(db, double_quoted_string.content()).into());
    }

    if node
      .parent()
      .is_some_and(|parent| ParenExpr::cast(parent).is_some())
    {
      let name = double_quoted_string.content();
      let symbol = (*pretype_resolve_name(db, file, node, name))?;
      return *pretype_evaluate_typ_symbol(db, symbol);
    }
  }

  if let Some(number) = NumberExpr::cast(node) {
    if let Some(integer) = number.int_value() {
      return Some(IntegerObj::new(db, integer).into());
    }
    return Some(FloatObj::new(db, number.raw_text()).into());
  }

  None
}

/// Evaluate an expression to a LazyTyp without eagerly evaluating referenced type symbols
pub fn pretype_evaluate_lazy_typ<'db>(
  db: &'db dyn Database,
  file: File,
  expr: &Expr,
) -> Option<LazyTyp<'db>> {
  let node = expr.syntax().clone();
  if let Some(ident) = IdentExpr::cast(node.clone()) {
    let name = ident.name();
    if let Some(symbol) = *pretype_resolve_name(db, file, node, name) {
      return Some(LazyTyp::lazy(symbol));
    }
  }

  pretype_evaluate_node(db, file, expr)
    .and_then(|obj| obj.as_typ())
    .map(LazyTyp::eager)
}
