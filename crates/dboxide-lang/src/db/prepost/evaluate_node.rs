use std::collections::BTreeMap;

use salsa::Database;

use crate::OStringObj;
use crate::ast::{
  AstNode, CallExpr, ClosureExpr, DqStringExpr, Expr, FuncDeclaration, IdentExpr, IndexExpr,
  InfixExpr, MatchExpr, NumberExpr, OqStringExpr, ParenExpr, PostfixExpr, SqStringExpr,
  TernaryExpr, TupleExpr,
};
use crate::db::pretype::evaluate_typ::evaluate_typ_symbol;
use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{
  AnyTyp, BoolObj, File, FloatObj, FuncObj, FuncTyp, IntegerObj, NullTyp, Obj, RuntimeObj,
  StaticTyp, StringObj, SumTyp, Typ,
};

use super::RuntimeScope;
use super::resolve_name::{resolve_name_posttype, resolve_name_pretype};

// Pretype: resolves identifiers to types/schemas only
pub fn evaluate_node_pretype<'db>(
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
    let symbol = (*resolve_name_pretype(db, file, node, name))?;
    return *evaluate_typ_symbol(db, symbol);
  }

  if let Some(index) = IndexExpr::cast(node.clone()) {
    let base_expr = index.base()?;
    let base = evaluate_node_pretype(db, file, &base_expr)?;
    if let Some(base_typ) = base.as_typ() {
      let lazy_args: Vec<LazyTyp<'db>> = index
        .indices()
        .filter_map(|arg| evaluate_lazy_typ_pretype(db, file, &arg))
        .collect();
      if let Some(instantiated) = base_typ.instantiate(db, lazy_args) {
        return Some(Obj::Typ(instantiated));
      }
    }
    let args: Vec<Obj<'db>> = index
      .indices()
      .filter_map(|arg| evaluate_node_pretype(db, file, &arg))
      .collect();
    if args.len() == 1 {
      return base.index(db, &args[0]);
    }
    return None;
  }

  if let Some(postfix) = PostfixExpr::cast(node.clone())
    && postfix.op().as_deref() == Some("?")
  {
    let inner = evaluate_node_pretype(db, file, &postfix.operand()?)?.as_typ()?;
    let null: Typ = NullTyp::new(db).into();
    return Some(Obj::Typ(SumTyp::create(db, vec![inner, null])));
  }

  if let Some(infix) = InfixExpr::cast(node.clone())
    && infix.op().as_deref() == Some("|")
  {
    let left = evaluate_node_pretype(db, file, &infix.left()?)?.as_typ()?;
    let right = evaluate_node_pretype(db, file, &infix.right()?)?.as_typ()?;
    return Some(Obj::Typ(SumTyp::create(db, vec![left, right])));
  }

  if let Some(single_quoted_string) = SqStringExpr::cast(node.clone()) {
    return Some(StringObj::new(db, single_quoted_string.content()).into());
  }

  if let Some(opaque_string) = OqStringExpr::cast(node.clone()) {
    return Some(OStringObj::new(db, opaque_string.content()).into());
  }

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
      let symbol = (*resolve_name_pretype(db, file, node, name))?;
      return *evaluate_typ_symbol(db, symbol);
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

pub fn evaluate_lazy_typ_pretype<'db>(
  db: &'db dyn Database,
  file: File,
  expr: &Expr,
) -> Option<LazyTyp<'db>> {
  let node = expr.syntax().clone();
  if let Some(ident) = IdentExpr::cast(node.clone()) {
    let name = ident.name();
    if let Some(symbol) = *resolve_name_pretype(db, file, node, name) {
      return Some(LazyTyp::lazy(symbol));
    }
  }
  evaluate_node_pretype(db, file, expr)
    .and_then(|obj| obj.as_typ())
    .map(LazyTyp::eager)
}

// Posttype: full evaluation including calls, closures, match, runtime scope
pub fn evaluate_node_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  expr: &Expr,
  scope: RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  let node = expr.syntax().clone();

  if let Some(ident) = IdentExpr::cast(node.clone()) {
    let name = ident.name();
    if let Some(obj) = scope.lookup(db, &name) {
      return Some(obj);
    }
    match name.as_str() {
      "true" => return Some(BoolObj::new(db, true).into()),
      "false" => return Some(BoolObj::new(db, false).into()),
      _ => {}
    }
    let symbol = (*resolve_name_posttype(db, file, node, name))?;
    return *evaluate_typ_symbol(db, symbol);
  }

  if let Some(index) = IndexExpr::cast(node.clone()) {
    let base_expr = index.base()?;
    let base = evaluate_node_posttype(db, file, &base_expr, scope)?;
    if let Some(base_typ) = base.as_typ() {
      let lazy_args: Vec<LazyTyp<'db>> = index
        .indices()
        .filter_map(|arg| evaluate_lazy_typ_posttype(db, file, &arg))
        .collect();
      if let Some(instantiated) = base_typ.instantiate(db, lazy_args) {
        return Some(Obj::Typ(instantiated));
      }
    }
    let args: Vec<Obj<'db>> = index
      .indices()
      .filter_map(|arg| evaluate_node_posttype(db, file, &arg, scope))
      .collect();
    if args.len() == 1 {
      return base.index(db, &args[0]);
    }
    return None;
  }

  if let Some(postfix) = PostfixExpr::cast(node.clone())
    && postfix.op().as_deref() == Some("?")
  {
    let inner = evaluate_node_posttype(db, file, &postfix.operand()?, scope)?.as_typ()?;
    let null: Typ = NullTyp::new(db).into();
    return Some(Obj::Typ(SumTyp::create(db, vec![inner, null])));
  }

  if let Some(infix) = InfixExpr::cast(node.clone()) {
    return evaluate_infix(db, file, &infix, scope);
  }

  if let Some(call) = CallExpr::cast(node.clone()) {
    return evaluate_call(db, file, &call, scope);
  }

  if let Some(closure) = ClosureExpr::cast(node.clone()) {
    return evaluate_closure(db, &closure, scope);
  }

  if let Some(ternary) = TernaryExpr::cast(node.clone()) {
    let cond = evaluate_node_posttype(db, file, &ternary.condition()?, scope)?;
    if let Some(b) = cond.as_bool(db) {
      if b {
        return evaluate_node_posttype(db, file, &ternary.then_expr()?, scope);
      } else {
        return evaluate_node_posttype(db, file, &ternary.else_expr()?, scope);
      }
    }
    return None;
  }

  if let Some(match_expr) = MatchExpr::cast(node.clone()) {
    let subject = evaluate_node_posttype(db, file, &match_expr.subject()?, scope)?;
    for arm in match_expr.arms() {
      let pattern = evaluate_node_posttype(db, file, &arm.pattern()?, scope)?;
      if obj_value_eq(db, &subject, &pattern) {
        return evaluate_node_posttype(db, file, &arm.result()?, scope);
      }
    }
    return None;
  }

  if let Some(single_quoted_string) = SqStringExpr::cast(node.clone()) {
    return Some(StringObj::new(db, single_quoted_string.content()).into());
  }
  if let Some(opaque_string) = OqStringExpr::cast(node.clone()) {
    return Some(OStringObj::new(db, opaque_string.content()).into());
  }
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
      let symbol = (*resolve_name_posttype(db, file, node, name))?;
      return *evaluate_typ_symbol(db, symbol);
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

pub fn evaluate_lazy_typ_posttype<'db>(
  db: &'db dyn Database,
  file: File,
  expr: &Expr,
) -> Option<LazyTyp<'db>> {
  let node = expr.syntax().clone();
  if let Some(ident) = IdentExpr::cast(node.clone()) {
    let name = ident.name();
    if let Some(symbol) = *resolve_name_posttype(db, file, node, name) {
      return Some(LazyTyp::lazy(symbol));
    }
  }
  evaluate_node_posttype(db, file, expr, RuntimeScope::empty(db))
    .and_then(|obj| obj.as_typ())
    .map(LazyTyp::eager)
}

fn evaluate_infix<'db>(
  db: &'db dyn Database,
  file: File,
  infix: &InfixExpr,
  scope: RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  match infix.op().as_deref() {
    Some("|") => {
      let left = evaluate_node_posttype(db, file, &infix.left()?, scope)?.as_typ()?;
      let right = evaluate_node_posttype(db, file, &infix.right()?, scope)?.as_typ()?;
      Some(Obj::Typ(SumTyp::create(db, vec![left, right])))
    }
    Some("@") => {
      let left = evaluate_node_posttype(db, file, &infix.left()?, scope)?;
      let field_name = infix.right()?.as_name()?;
      left.lookup_field(db, &field_name)
    }
    Some(".") => {
      let left = evaluate_node_posttype(db, file, &infix.left()?, scope)?;
      let member_name = infix.right()?.as_name()?;
      if let Some(typ) = left.as_typ()
        && let Some(lazy) = typ.lookup_namespace_member(db, &member_name)
        && let Some(resolved) = lazy.resolve(db)
      {
        return Some(Obj::Typ(resolved));
      }
      left.lookup_field(db, &member_name)
    }
    Some(op_str) => {
      let left = evaluate_node_posttype(db, file, &infix.left()?, scope)?;
      let left_typ = left.get_typ(db);
      let func_obj = *left_typ.runtime_vtable(db).get(op_str)?;
      let right = evaluate_node_posttype(db, file, &infix.right()?, scope)?;
      call_func_obj(db, file, func_obj, vec![left, right], scope)
    }
    None => None,
  }
}

fn evaluate_call<'db>(
  db: &'db dyn Database,
  file: File,
  call: &CallExpr,
  scope: RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  let callee_expr = call.callee()?;
  let callee = evaluate_node_posttype(db, file, &callee_expr, scope)?;
  let args: Vec<Obj<'db>> = call
    .args()
    .filter_map(|arg| evaluate_node_posttype(db, file, &arg, scope))
    .collect();
  if let Some(typ) = callee.as_typ()
    && typ.call_typ(db).is_some()
  {
    return typ
      .construct(db, args.iter().filter_map(|a| a.as_typ()).collect())
      .map(Obj::Typ);
  }
  if let Obj::Func(func_obj) = callee {
    return call_func_obj(db, file, func_obj, args, scope);
  }
  None
}

fn call_func_obj<'db>(
  db: &'db dyn Database,
  file: File,
  func_obj: FuncObj<'db>,
  args: Vec<Obj<'db>>,
  call_scope: RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  // Use the captured (lexical) scope as parent, falling back to call-site scope
  let defining_scope = func_obj.captured_scope(db).unwrap_or(call_scope);
  let source = func_obj.source(db).as_ref()?.clone();
  if let Some(closure) = ClosureExpr::cast(source.clone()) {
    let param_names = extract_closure_params(&closure);
    let mut bindings = BTreeMap::new();
    for (name, arg) in param_names.into_iter().zip(args) {
      bindings.insert(name, arg);
    }
    let child_scope = defining_scope.child(db, bindings);
    let body = closure.body()?;
    return evaluate_node_posttype(db, file, &body, child_scope);
  }
  if let Some(func_declaration) = FuncDeclaration::cast(source) {
    let mut bindings = BTreeMap::new();
    if let Some(params_node) = func_declaration.params() {
      for (param, arg) in params_node.params().zip(args) {
        if let Some(ident) = param.name() {
          bindings.insert(ident.name(), arg);
        }
      }
    }
    let child_scope = defining_scope.child(db, bindings);
    let body = func_declaration.body()?;
    let body_expr = Expr::cast(body.syntax().clone())?;
    return evaluate_node_posttype(db, file, &body_expr, child_scope);
  }
  None
}

fn extract_closure_params(closure: &ClosureExpr) -> Vec<String> {
  let Some(params) = closure.params() else {
    return Vec::new();
  };
  if let Some(tuple) = TupleExpr::cast(params.syntax().clone()) {
    tuple.items().filter_map(|item| item.as_name()).collect()
  } else if let Some(ident) = IdentExpr::cast(params.syntax().clone()) {
    vec![ident.name()]
  } else {
    Vec::new()
  }
}

fn evaluate_closure<'db>(
  db: &'db dyn Database,
  closure: &ClosureExpr,
  scope: RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  let param_count = if let Some(params) = closure.params() {
    if let Some(tuple) = TupleExpr::cast(params.syntax().clone()) {
      tuple.items().count()
    } else {
      1
    }
  } else {
    0
  };
  let any_typ = LazyTyp::eager(AnyTyp::new(db).into());
  let params = vec![any_typ; param_count];
  let signature = FuncSignature::new(db, params, any_typ);
  let func_typ = FuncTyp::new(db, signature);
  let func_obj = FuncObj::new(
    db,
    "<closure>".to_string(),
    func_typ,
    Some(closure.syntax().clone()),
    Some(scope),
  );
  Some(Obj::Func(func_obj))
}

fn obj_value_eq<'db>(db: &'db dyn Database, a: &Obj<'db>, b: &Obj<'db>) -> bool {
  match (a, b) {
    (Obj::Bool(a), Obj::Bool(b)) => a.value(db) == b.value(db),
    (Obj::Integer(a), Obj::Integer(b)) => a.value(db) == b.value(db),
    (Obj::Float(a), Obj::Float(b)) => a.value(db) == b.value(db),
    (Obj::String(a), Obj::String(b)) => a.value(db) == b.value(db),
    _ => a == b,
  }
}
