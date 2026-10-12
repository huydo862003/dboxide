use std::collections::BTreeMap;

use salsa::Database;

use crate::OStringObj;
use crate::ast::{
  AstNode, CallExpr, ClosureExpr, DqStringExpr, Expr, IdentExpr, IndexExpr, InfixExpr, NumberExpr,
  OqStringExpr, ParenExpr, PostfixExpr, SqStringExpr, TupleExpr,
};
use crate::db::pretype::pretype_evaluate::pretype_evaluate_typ_symbol::evaluate_typ_symbol;
use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{
  BoolObj, File, FloatObj, FuncObj, FuncTyp, IntegerObj, NullTyp, Obj, RuntimeObj, StaticTyp,
  StringObj, SumTyp, Typ,
};

use super::posttype_resolve_name::resolve_name;
use super::runtime_scope::RuntimeScope;

// evaluate an expression node into an Obj using posttype scope
pub fn evaluate_node<'db>(db: &'db dyn Database, file: File, expr: &Expr) -> Option<Obj<'db>> {
  evaluate_with_scope(db, file, expr, &RuntimeScope::empty())
}

// evaluate with a runtime scope (for closure bodies with bound params)
pub fn evaluate_with_scope<'db>(
  db: &'db dyn Database,
  file: File,
  expr: &Expr,
  scope: &RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  let node = expr.syntax().clone();

  // identifier: check runtime scope first (self, super, closure params), then literals, then name resolution
  if let Some(ident) = IdentExpr::cast(node.clone()) {
    let name = ident.name();
    if let Some(obj) = scope.lookup(&name) {
      return Some(obj);
    }
    match name.as_str() {
      "true" => return Some(BoolObj::new(db, true).into()),
      "false" => return Some(BoolObj::new(db, false).into()),
      _ => {}
    }
    let symbol = (*resolve_name(db, file, node, name))?;
    return *evaluate_typ_symbol(db, symbol);
  }

  // index: type instantiation or runtime indexing
  if let Some(index) = IndexExpr::cast(node.clone()) {
    let base_expr = index.base()?;
    let base = evaluate_with_scope(db, file, &base_expr, scope)?;
    if let Some(base_typ) = base.as_typ() {
      let lazy_args: Vec<LazyTyp<'db>> = index
        .indices()
        .filter_map(|arg| evaluate_lazy_typ(db, file, &arg))
        .collect();
      if let Some(instantiated) = base_typ.instantiate(db, lazy_args) {
        return Some(Obj::Typ(instantiated));
      }
    }
    let args: Vec<Obj<'db>> = index
      .indices()
      .filter_map(|arg| evaluate_with_scope(db, file, &arg, scope))
      .collect();
    if args.len() == 1 {
      return base.index(db, &args[0]);
    }
    return None;
  }

  // postfix `?`: nullable type T?
  if let Some(postfix) = PostfixExpr::cast(node.clone())
    && postfix.op().as_deref() == Some("?")
  {
    let inner = evaluate_with_scope(db, file, &postfix.operand()?, scope)?.as_typ()?;
    let null: Typ = NullTyp::new(db).into();
    return Some(Obj::Typ(SumTyp::create(db, vec![inner, null])));
  }

  // infix operators
  if let Some(infix) = InfixExpr::cast(node.clone()) {
    return evaluate_infix(db, file, &infix, scope);
  }

  // function call: callee(args)
  if let Some(call) = CallExpr::cast(node.clone()) {
    return evaluate_call(db, file, &call, scope);
  }

  // closure: (params) => body - creates a FuncObj capturing the current scope
  if let Some(closure) = ClosureExpr::cast(node.clone()) {
    return evaluate_closure(db, &closure);
  }

  // string literals
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
      let symbol = (*resolve_name(db, file, node, name))?;
      return *evaluate_typ_symbol(db, symbol);
    }
  }

  // numbers
  if let Some(number) = NumberExpr::cast(node) {
    if let Some(integer) = number.int_value() {
      return Some(IntegerObj::new(db, integer).into());
    }
    return Some(FloatObj::new(db, number.raw_text()).into());
  }

  None
}

fn evaluate_infix<'db>(
  db: &'db dyn Database,
  file: File,
  infix: &InfixExpr,
  scope: &RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  match infix.op().as_deref() {
    // type union: T | U
    Some("|") => {
      let left = evaluate_with_scope(db, file, &infix.left()?, scope)?.as_typ()?;
      let right = evaluate_with_scope(db, file, &infix.right()?, scope)?.as_typ()?;
      Some(Obj::Typ(SumTyp::create(db, vec![left, right])))
    }
    // member access: x@y
    Some("@") => {
      let left = evaluate_with_scope(db, file, &infix.left()?, scope)?;
      let field_name = infix.right()?.as_name()?;
      left.lookup_field(db, &field_name)
    }
    // namespace traversal: x.y
    Some(".") => {
      let left = evaluate_with_scope(db, file, &infix.left()?, scope)?;
      let member_name = infix.right()?.as_name()?;
      if let Some(typ) = left.as_typ()
        && let Some(lazy) = typ.lookup_namespace_member(db, &member_name)
        && let Some(resolved) = lazy.resolve(db)
      {
        return Some(Obj::Typ(resolved));
      }
      left.lookup_field(db, &member_name)
    }
    // operator dispatch via vtable on LHS type
    Some(op_str) => {
      let left = evaluate_with_scope(db, file, &infix.left()?, scope)?;
      let left_typ = left.get_typ(db);
      let func_obj = *left_typ.runtime_vtable(db).get(op_str)?;
      let right = evaluate_with_scope(db, file, &infix.right()?, scope)?;
      call_func_obj(db, file, func_obj, vec![left, right], scope)
    }
    None => None,
  }
}

fn evaluate_call<'db>(
  db: &'db dyn Database,
  file: File,
  call: &CallExpr,
  scope: &RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  let callee_expr = call.callee()?;
  let callee = evaluate_with_scope(db, file, &callee_expr, scope)?;

  let args: Vec<Obj<'db>> = call
    .args()
    .filter_map(|arg| evaluate_with_scope(db, file, &arg, scope))
    .collect();

  // type-level call (constructor)
  if let Some(typ) = callee.as_typ()
    && typ.call_typ(db).is_some()
  {
    return typ
      .construct(db, args.iter().filter_map(|a| a.as_typ()).collect())
      .map(Obj::Typ);
  }

  // FuncObj call (closure or named function)
  if let Obj::Func(func_obj) = callee {
    return call_func_obj(db, file, func_obj, args, scope);
  }

  None
}

// call a FuncObj: extract params from source node, bind args, evaluate body
fn call_func_obj<'db>(
  db: &'db dyn Database,
  file: File,
  func_obj: FuncObj<'db>,
  args: Vec<Obj<'db>>,
  parent_scope: &RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  let source = func_obj.source(db).as_ref()?.clone();

  // closure: source is ClosureExpr, extract params and body from it
  if let Some(closure) = ClosureExpr::cast(source.clone()) {
    let param_names = extract_closure_params(&closure);
    let mut bindings = BTreeMap::new();
    for (name, arg) in param_names.into_iter().zip(args) {
      bindings.insert(name, arg);
    }
    let child_scope = parent_scope.child(bindings);
    let body = closure.body()?;
    return evaluate_with_scope(db, file, &body, &child_scope);
  }

  // named function: source is FuncDeclaration, extract params and body
  if let Some(func_decl) = crate::ast::FuncDeclaration::cast(source) {
    let mut bindings = BTreeMap::new();
    if let Some(params_node) = func_decl.params() {
      for (param, arg) in params_node.params().zip(args) {
        if let Some(ident) = param.name() {
          bindings.insert(ident.name(), arg);
        }
      }
    }
    let child_scope = parent_scope.child(bindings);
    let body = func_decl.body()?;
    let body_expr = Expr::cast(body.syntax().clone())?;
    return evaluate_with_scope(db, file, &body_expr, &child_scope);
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

// creates a FuncObj from a closure expression
fn evaluate_closure<'db>(db: &'db dyn Database, closure: &ClosureExpr) -> Option<Obj<'db>> {
  let param_count = if let Some(params) = closure.params() {
    if let Some(tuple) = TupleExpr::cast(params.syntax().clone()) {
      tuple.items().count()
    } else {
      1
    }
  } else {
    0
  };

  let any_typ = LazyTyp::eager(crate::db::types::AnyTyp::new(db).into());
  let params = vec![any_typ; param_count];
  let signature = FuncSignature::new(db, params, any_typ);
  let func_typ = FuncTyp::new(db, signature);
  let func_obj = FuncObj::new(
    db,
    "<closure>".to_string(),
    func_typ,
    Some(closure.syntax().clone()),
  );
  Some(Obj::Func(func_obj))
}

// lazy evaluation: defer symbol resolution when possible
pub fn evaluate_lazy_typ<'db>(
  db: &'db dyn Database,
  file: File,
  expr: &Expr,
) -> Option<LazyTyp<'db>> {
  let node = expr.syntax().clone();
  if let Some(ident) = IdentExpr::cast(node.clone()) {
    let name = ident.name();
    if let Some(symbol) = *resolve_name(db, file, node, name) {
      return Some(LazyTyp::lazy(symbol));
    }
  }
  evaluate_node(db, file, expr)
    .and_then(|obj| obj.as_typ())
    .map(LazyTyp::eager)
}
