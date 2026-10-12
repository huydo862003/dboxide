use std::collections::BTreeMap;

use salsa::Database;

use crate::OStringObj;
use crate::ast::{
  AstNode, CallExpr, ClosureExpr, DqStringExpr, ExistsExpr, Expr, ForallExpr, FuncDeclaration,
  IdentExpr, IndexExpr, InfixExpr, MatchExpr, NumberExpr, OqStringExpr, ParenExpr, PostfixExpr,
  SqStringExpr, TernaryExpr, TupleExpr,
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
    let condition = evaluate_node_posttype(db, file, &ternary.condition()?, scope)?;
    if let Some(bool) = condition.as_bool(db) {
      if bool {
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

  if let Some(forall) = ForallExpr::cast(node.clone()) {
    return evaluate_quantifier(db, file, &forall, scope, true);
  }

  if let Some(exists) = ExistsExpr::cast(node.clone()) {
    return evaluate_quantifier(db, file, &exists, scope, false);
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

/// `forall c of collection { body }` / `exists c of collection { body }`
/// `all = true` -> forall (all items must satisfy body)
/// `all = false` -> exists (at least one item must satisfy body)
fn evaluate_quantifier<'db, T>(
  db: &'db dyn Database,
  file: File,
  expr: &T,
  scope: RuntimeScope<'db>,
  all: bool,
) -> Option<Obj<'db>>
where
  T: QuantifierExpr<'db>,
{
  let binding_name = expr.quantifier_binding()?;
  let collection = evaluate_node_posttype(db, file, &expr.quantifier_collection()?, scope)?;
  let body_expr = expr.quantifier_body_expr()?;

  let items = match collection {
    Obj::List(list) => list.items(db).clone(),
    _ => return None,
  };

  for item in &items {
    let child_scope = scope.child(db, BTreeMap::from([(binding_name.clone(), *item)]));
    let result = evaluate_node_posttype(db, file, &body_expr, child_scope);
    let is_true = matches!(result, Some(Obj::Bool(b)) if b.value(db));
    if all && !is_true {
      return Some(BoolObj::new(db, false).into());
    }
    if !all && is_true {
      return Some(BoolObj::new(db, true).into());
    }
  }

  Some(BoolObj::new(db, all).into())
}

trait QuantifierExpr<'db> {
  fn quantifier_binding(&self) -> Option<String>;
  fn quantifier_collection(&self) -> Option<Expr>;
  fn quantifier_body_expr(&self) -> Option<Expr>;
}

impl<'db> QuantifierExpr<'db> for ForallExpr {
  fn quantifier_binding(&self) -> Option<String> {
    self.binding()?.as_name()
  }

  fn quantifier_collection(&self) -> Option<Expr> {
    self.collection()
  }

  fn quantifier_body_expr(&self) -> Option<Expr> {
    self.body()?.fields().next()?.args().next()?.expr()
  }
}

impl<'db> QuantifierExpr<'db> for ExistsExpr {
  fn quantifier_binding(&self) -> Option<String> {
    self.binding()?.as_name()
  }

  fn quantifier_collection(&self) -> Option<Expr> {
    self.collection()
  }

  fn quantifier_body_expr(&self) -> Option<Expr> {
    self.body()?.fields().next()?.args().next()?.expr()
  }
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
    Some(op_string) => {
      let left = evaluate_node_posttype(db, file, &infix.left()?, scope)?;
      let left_typ = left.get_typ(db);
      let func_obj = *left_typ.runtime_vtable(db).get(op_string)?;
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

#[cfg(test)]
mod tests {
  use std::collections::{BTreeMap, HashMap};

  use itertools::Either;
  use salsa::Database;

  use crate::ast::SourceFile;
  use crate::ast::{AstNode, BlockElementDeclaration};
  use crate::db::DboxideDatabase;
  use crate::db::prepost::RuntimeScope;
  use crate::db::prepost::evaluate_node::evaluate_node_posttype;
  use crate::db::test_utils::*;
  use crate::db::types::ListObj;
  use crate::db::types::interned::symbol::VirtualModuleKind;
  use crate::parse_file::parse_file;
  use crate::{IntegerObj, Obj};

  fn metatype() -> HashMap<String, Either<VirtualModuleKind, crate::db::types::File>> {
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )])
  }

  #[test]
  fn evaluate_node_handles_string_literal() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let file = create_file(
        db,
        "test.dbml",
        r#"
use * from 'metatype'

type Project [element] {
  note string? [attribute]
}

Project myproject {
  note: 'hello'
}
"#,
        HashMap::from([(
          "metatype".to_string(),
          Either::Left(VirtualModuleKind::Metatype),
        )]),
      );

      let root = parse_file(db, file).ast(db).value(db).node.clone();
      let source_file = SourceFile::cast(root).unwrap();

      for element in source_file.element_declarations() {
        let Some(block) = BlockElementDeclaration::cast(element.syntax().clone()) else {
          continue;
        };
        let Some(body) = block.body() else { continue };
        for attribute in body.attributes() {
          if let Some(value) = attribute.value()
            && let Some(expr) = value.expr()
          {
            let result = evaluate_node_posttype(db, file, &expr, RuntimeScope::empty(db));
            assert!(matches!(result, Some(Obj::String(_))));
            return;
          }
        }
      }
      panic!("did not find the string expression to evaluate");
    }

    let db = DboxideDatabase::default();
    run(&db);
  }

  #[test]
  fn ternary_evaluates_true_branch() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let file = create_file(
        db,
        "test.dbml",
        r#"
use * from 'metatype'

type Project [element] {
  note string? [attribute]
}

Project myproject {
  note: true ? 1 : 2
}
"#,
        HashMap::from([(
          "metatype".to_string(),
          Either::Left(VirtualModuleKind::Metatype),
        )]),
      );

      let root = parse_file(db, file).ast(db).value(db).node.clone();
      let source_file = SourceFile::cast(root).unwrap();

      for element in source_file.element_declarations() {
        let Some(block) = BlockElementDeclaration::cast(element.syntax().clone()) else {
          continue;
        };
        let Some(body) = block.body() else { continue };
        for attribute in body.attributes() {
          if let Some(value) = attribute.value()
            && let Some(expr) = value.expr()
          {
            let result = evaluate_node_posttype(db, file, &expr, RuntimeScope::empty(db));
            assert!(
              matches!(result, Some(Obj::Integer(i)) if i.value(db) == 1),
              "expected ternary to evaluate to 1"
            );
            return;
          }
        }
      }
      panic!("did not find the ternary expression to evaluate");
    }

    let db = DboxideDatabase::default();
    run(&db);
  }

  #[test]
  fn match_evaluates_matching_arm() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let file = create_file(
        db,
        "test.dbml",
        r#"
use * from 'metatype'

type Project [element] {
  note string? [attribute]
}

Project myproject {
  note: match true { true => 1 false => 2 }
}
"#,
        HashMap::from([(
          "metatype".to_string(),
          Either::Left(VirtualModuleKind::Metatype),
        )]),
      );

      let root = parse_file(db, file).ast(db).value(db).node.clone();
      let source_file = SourceFile::cast(root).unwrap();

      for element in source_file.element_declarations() {
        let Some(block) = BlockElementDeclaration::cast(element.syntax().clone()) else {
          continue;
        };
        let Some(body) = block.body() else { continue };
        for attribute in body.attributes() {
          if let Some(value) = attribute.value()
            && let Some(expr) = value.expr()
          {
            let result = evaluate_node_posttype(db, file, &expr, RuntimeScope::empty(db));
            assert!(
              matches!(result, Some(Obj::Integer(i)) if i.value(db) == 1),
              "expected match to evaluate to 1"
            );
            return;
          }
        }
      }
      panic!("did not find the match expression to evaluate");
    }

    let db = DboxideDatabase::default();
    run(&db);
  }

  #[test]
  fn scope_variable_lookup_resolves_binding() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let file = create_file(
        db,
        "test.dbml",
        r#"
use * from 'metatype'

type Foo [element] {
  note string? [attribute]
}

Foo f {
  note: x
}
"#,
        HashMap::from([(
          "metatype".to_string(),
          Either::Left(VirtualModuleKind::Metatype),
        )]),
      );

      let root = parse_file(db, file).ast(db).value(db).node.clone();
      let source_file = SourceFile::cast(root).unwrap();

      // Build a scope with x = 42
      let x_value = IntegerObj::new(db, 42).into();
      let scope = RuntimeScope::with_bindings(db, BTreeMap::from([("x".to_string(), x_value)]));

      for element in source_file.element_declarations() {
        let Some(block) = BlockElementDeclaration::cast(element.syntax().clone()) else {
          continue;
        };
        let Some(body) = block.body() else { continue };
        for attribute in body.attributes() {
          if let Some(value) = attribute.value()
            && let Some(expr) = value.expr()
          {
            let result = evaluate_node_posttype(db, file, &expr, scope);
            assert!(
              matches!(result, Some(Obj::Integer(i)) if i.value(db) == 42),
              "expected scope lookup to resolve x = 42, got (non-Debug type)"
            );
            return;
          }
        }
      }
      panic!("did not find the expression to evaluate");
    }

    let db = DboxideDatabase::default();
    run(&db);
  }

  #[test]
  fn closure_evaluates_to_func_obj() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let file = create_file(
        db,
        "test.dbml",
        r#"
use * from 'metatype'

type Foo [element] {
  note string? [attribute]
}

Foo f {
  note: () => 99
}
"#,
        HashMap::from([(
          "metatype".to_string(),
          Either::Left(VirtualModuleKind::Metatype),
        )]),
      );

      let root = parse_file(db, file).ast(db).value(db).node.clone();
      let source_file = SourceFile::cast(root).unwrap();

      for element in source_file.element_declarations() {
        let Some(block) = BlockElementDeclaration::cast(element.syntax().clone()) else {
          continue;
        };
        let Some(body) = block.body() else { continue };
        for attribute in body.attributes() {
          if let Some(value) = attribute.value()
            && let Some(expr) = value.expr()
          {
            let result = evaluate_node_posttype(db, file, &expr, RuntimeScope::empty(db));
            assert!(
              matches!(result, Some(Obj::Func(_))),
              "expected closure to evaluate to FuncObj, got (non-Debug type)"
            );
            return;
          }
        }
      }
      panic!("did not find the closure expression to evaluate");
    }

    let db = DboxideDatabase::default();
    run(&db);
  }

  fn eval_quantifier_src<'db>(
    db: &'db dyn Database,
    src: &str,
    items: Vec<Obj<'db>>,
  ) -> Option<Obj<'db>> {
    let file = create_file(db, "test.dbml", src, HashMap::new());
    let list = ListObj::new(db, items);
    let scope =
      RuntimeScope::with_bindings(db, BTreeMap::from([("nums".to_string(), Obj::List(list))]));
    let root = parse_file(db, file).ast(db).value(db).node.clone();
    let source = SourceFile::cast(root).unwrap();
    for func in source.func_declarations() {
      if let Some(body) = func.body() {
        for field in body.fields() {
          if let Some(arg) = field.args().next()
            && let Some(expr) = arg.expr()
          {
            return evaluate_node_posttype(db, file, &expr, scope);
          }
        }
      }
    }
    panic!("no quantifier expression found");
  }

  #[test]
  fn forall_over_list_all_true_returns_true() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let items = vec![
        Obj::Integer(IntegerObj::new(db, 1)),
        Obj::Integer(IntegerObj::new(db, 2)),
      ];
      let result = eval_quantifier_src(db, "fn f(): bool { forall n of nums { true } }", items);
      assert!(
        matches!(result, Some(Obj::Bool(b)) if b.value(db)),
        "forall all-true list should return true"
      );
    }
    run(&DboxideDatabase::default());
  }

  #[test]
  fn forall_over_list_one_false_returns_false() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let items = vec![Obj::Integer(IntegerObj::new(db, 1))];
      let result = eval_quantifier_src(db, "fn f(): bool { forall n of nums { false } }", items);
      assert!(
        matches!(result, Some(Obj::Bool(b)) if !b.value(db)),
        "forall with false body should return false"
      );
    }
    run(&DboxideDatabase::default());
  }

  #[test]
  fn forall_over_empty_list_returns_true() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let result = eval_quantifier_src(db, "fn f(): bool { forall n of nums { false } }", vec![]);
      assert!(
        matches!(result, Some(Obj::Bool(b)) if b.value(db)),
        "forall over empty list is vacuously true"
      );
    }
    run(&DboxideDatabase::default());
  }

  #[test]
  fn exists_over_list_one_true_returns_true() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let items = vec![Obj::Integer(IntegerObj::new(db, 1))];
      let result = eval_quantifier_src(db, "fn f(): bool { exists n of nums { true } }", items);
      assert!(
        matches!(result, Some(Obj::Bool(b)) if b.value(db)),
        "exists with one match should return true"
      );
    }
    run(&DboxideDatabase::default());
  }

  #[test]
  fn exists_over_empty_list_returns_false() {
    #[salsa::tracked]
    fn run(db: &dyn Database) {
      let result = eval_quantifier_src(db, "fn f(): bool { exists n of nums { true } }", vec![]);
      assert!(
        matches!(result, Some(Obj::Bool(b)) if !b.value(db)),
        "exists over empty list is false"
      );
    }
    run(&DboxideDatabase::default());
  }
}
