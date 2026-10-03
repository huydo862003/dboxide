use std::collections::BTreeMap;

use salsa::{Accumulator, Database, tracked};

use crate::ast::{
  AstNode, BlockElementDeclaration, BlockElementDeclarationBody, CallExpr, ClosureExpr, ColorExpr,
  CsvExpr, DqStringExpr, ElementDeclaration, ElementFieldDeclaration, Expr, FuncDeclaration,
  GetDeclaration, IdentExpr, IndexExpr, InfixExpr, InlineElementDeclaration, ListExpr, MatchExpr,
  NumberExpr, OqStringExpr, ParenExpr, PostfixExpr, PrefixExpr, QuantifierExpr, SettingListItem,
  SourceFile, SqStringExpr, TernaryExpr, TqStringExpr, TupleExpr,
};
use crate::db::posttype_analyze::{posttype_get_scope_members, posttype_resolve_referee};
use crate::db::pretype_analyze::get_declaration_typ;
use crate::db::pretype_evaluate_typ::pretype_evaluate_typ_symbol;
use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::SymbolCategoryKind;
use crate::db::types::{
  ArgFieldDefinition, BoolObj, ColorObj, Diagnostics, ElementObj, ElementTyp, File, FloatObj,
  FuncObj, FuncTyp, IntegerObj, ListObj, ListTyp, NullObj, NullTyp, Obj, ObjTyp, RuntimeObj,
  RuntimeScope, StaticTyp, StringObj, SumTyp, Typ,
};
use crate::diagnostics::DiagnosticKind;
use crate::parse_file::parse_file;
use crate::{OStringObj, extract_declaration_name, is_declaration_slot};
use crate::{SymbolKind, get_node_static_scope};

/// Evaluates all element instances in a file into ElementObj values
#[tracked]
pub fn evaluate_file<'db>(db: &'db dyn Database, file: File) -> Vec<ElementObj<'db>> {
  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return Vec::new();
  };

  let scope = RuntimeScope::empty(db, file);
  let mut elements = Vec::new();
  for element_declaration in source_file.element_declarations() {
    if element_declaration.is_typ_declaration() {
      continue;
    }
    if let Some(obj) = evaluate_element(db, scope, &element_declaration) {
      elements.push(obj);
    }
  }
  elements
}

/// Evaluates a single element declaration into an ElementObj
pub fn evaluate_element<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  element: &ElementDeclaration,
) -> Option<ElementObj<'db>> {
  let Some(Typ::Element(element_typ)) =
    *get_declaration_typ(db, scope.file(db), element.syntax().clone())
  else {
    return None;
  };

  let mut fields = BTreeMap::new();

  // Label and schema
  if let Some(target) = element.target_name() {
    let full_text = target.text().trim().to_string();
    if !full_text.is_empty() {
      if let Some(name_fragment) = target.name() {
        fields.insert(
          "name".to_string(),
          StringObj::new(db, name_fragment.name().to_string()).into(),
        );
      }
      if !target.schema().is_empty()
        && let Some(schema) = target.schema().into_iter().next()
      {
        fields.insert(
          "schema".to_string(),
          StringObj::new(db, schema.name().to_string()).into(),
        );
      }
    }
  }

  // Alias
  if let Some(alias) = element.alias() {
    let alias_text = alias.text().trim().to_string();
    if !alias_text.is_empty() {
      fields.insert("alias".to_string(), StringObj::new(db, alias_text).into());
    }
  }

  // Header settings
  if let Some(setting_list) = element.setting_list() {
    for item in setting_list.items() {
      let Some(name_node) = item.name() else {
        continue;
      };
      if let Some(obj) = evaluate_setting_value(db, scope, &item) {
        fields.insert(name_node.name(), obj);
      }
    }
  }

  // Inline body: single field line (e.g. `Ref: from op to [settings]`)
  if let Some(inline) = InlineElementDeclaration::cast(element.syntax().clone())
    && let Some(field_line) = inline.field()
    && let Some(field_typ) = element_typ.field_line_typ(db)
    && let Some(obj) = evaluate_field_line(db, scope, &field_line, field_typ)
  {
    let collection_name = element_typ
      .collection_field_name(db, field_typ.name(db))
      .unwrap_or_else(|| "fields".to_string());
    fields.insert(collection_name, ListObj::new(db, vec![obj]).into());
  }

  // Block body: attributes, field lines, sub-elements
  if let Some(block) = BlockElementDeclaration::cast(element.syntax().clone())
    && let Some(body) = block.body()
  {
    // Attributes
    for attribute in body.attributes() {
      let Some(attribute_name) = attribute
        .name()
        .and_then(|name_node| name_node.name())
        .and_then(|expr| expr.as_name())
      else {
        continue;
      };
      if let Some(expr) = attribute.value().and_then(|value| value.expr())
        && let Some(obj) = posttype_evaluate_node(db, scope, &expr)
      {
        fields.insert(attribute_name, obj);
      }
    }

    // Field lines
    if let Some(field_typ) = element_typ.field_line_typ(db) {
      let field_line_objs: Vec<_> = body
        .fields()
        .filter_map(|field_line| evaluate_field_line(db, scope, &field_line, field_typ))
        .collect();
      let collection_name = element_typ
        .collection_field_name(db, field_typ.name(db))
        .unwrap_or_else(|| "fields".to_string());
      fields.insert(collection_name, ListObj::new(db, field_line_objs).into());
    }

    // Sub-elements grouped by type
    let mut sub_element_groups: BTreeMap<String, Vec<Obj<'db>>> = BTreeMap::new();
    for nested in body.element_declarations() {
      let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
        continue;
      };
      if element.is_typ_declaration() {
        continue;
      }
      if let Some(obj) = evaluate_element(db, scope, &element) {
        let typ_name = match obj.element_typ(db) {
          Typ::Element(element) => element.name(db).clone(),
          _ => continue,
        };
        let collection_name = element_typ
          .collection_field_name(db, &typ_name)
          .unwrap_or_else(|| "children".to_string());
        sub_element_groups
          .entry(collection_name)
          .or_default()
          .push(Obj::Element(obj));
      }
    }
    for (name, elements) in sub_element_groups {
      fields.insert(name, ListObj::new(db, elements).into());
    }
  }

  Some(ElementObj::new(db, Typ::Element(element_typ), fields))
}

/// Evaluates an expression in posttype context
pub fn posttype_evaluate_node<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  expr: &Expr,
) -> Option<Obj<'db>> {
  let node = expr.syntax().clone();

  /* Ident */
  if let Some(ident) = IdentExpr::cast(node.clone()) {
    let name = ident.name();
    if let Some(obj) = scope.lookup(db, &name) {
      return Some(obj);
    }
    if !matches!(name.as_str(), "true" | "false" | "null") {
      let result = (*posttype_resolve_referee(db, scope.file(db), node))?;
      let Some(symbol) = result.symbol else {
        // Type-only resolution (no symbol): cannot evaluate, fall through
        return None;
      };
      // Element instances: evaluate the declaration to get the ElementObj
      if let SymbolKind::UserElement(element_file, element_node, _) = symbol.kind(db) {
        if let Some(element_declaration) = ElementDeclaration::cast(element_node.clone()) {
          let element_scope = RuntimeScope::empty(db, *element_file);
          return evaluate_element(db, element_scope, &element_declaration).map(Obj::Element);
        }
        if let Some(field_declaration) = ElementFieldDeclaration::cast(element_node.clone())
          && let Some(field_typ) = resolve_field_line_typ(db, *element_file, &field_declaration)
        {
          let element_scope = RuntimeScope::empty(db, *element_file);
          return evaluate_field_line(db, element_scope, &field_declaration, field_typ);
        }
      }
      return *pretype_evaluate_typ_symbol(db, symbol);
    }
  }

  /* Structural */
  if let Some(index) = IndexExpr::cast(node.clone()) {
    let base_expr = index.base()?;
    let base = posttype_evaluate_node(db, scope, &base_expr)?;
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
        .filter_map(|arg| posttype_evaluate_lazy_typ(db, scope, &arg))
        .collect();
      if let Some(instantiated) = base_typ.instantiate(db, lazy_args) {
        return Some(Obj::Typ(instantiated));
      }
    }
    // `value[key]`: runtime index access on a non-type value
    let args: Vec<Obj<'db>> = index
      .indices()
      .filter_map(|arg| posttype_evaluate_node(db, scope, &arg))
      .collect();
    if args.len() == 1 {
      return base.index(db, &args[0]);
    }
    return None;
  }

  if let Some(paren) = ParenExpr::cast(node.clone()) {
    // `(expr)`: transparent grouping, evaluates inner expression
    return paren
      .inner()
      .and_then(|inner| posttype_evaluate_node(db, scope, &inner));
  }

  /* Operators */
  if let Some(prefix) = PrefixExpr::cast(node.clone()) {
    let op = prefix.op()?;
    let operand_obj = posttype_evaluate_node(db, scope, &prefix.operand()?);
    if let Some(operand) = operand_obj {
      if let Some(func_obj) = operand.get_typ(db).lookup_runtime_op(db, op.as_str()) {
        return call_func_obj(db, func_obj, vec![operand], scope);
      }
      match op.as_str() {
        "!" => return Some(BoolObj::new(db, !operand.as_bool(db).unwrap_or(false)).into()),
        "-" => {
          if let Obj::Integer(int) = operand {
            return Some(IntegerObj::new(db, -int.value(db)).into());
          }
          if let Obj::Float(float) = operand {
            let value: f64 = float.value(db).parse().unwrap_or(0.0);
            return Some(FloatObj::new(db, (-value).to_string()).into());
          }
        }
        _ => {}
      }
    } else if op == "!" {
      return Some(BoolObj::new(db, true).into());
    }
    return None;
  }

  if let Some(postfix) = PostfixExpr::cast(node.clone())
    && postfix.has_op("?")
  {
    let inner = posttype_evaluate_node(db, scope, &postfix.operand()?)?.as_typ()?;
    let null: Typ = NullTyp::new(db).into();
    return Some(Obj::Typ(SumTyp::create(db, vec![inner, null])));
  }

  if let Some(infix) = InfixExpr::cast(node.clone()) {
    return evaluate_infix(db, &infix, scope);
  }

  /* Calls and closures */
  if let Some(call) = CallExpr::cast(node.clone()) {
    return evaluate_call(db, &call, scope);
  }

  if let Some(closure) = ClosureExpr::cast(node.clone()) {
    return evaluate_closure(db, &closure, scope);
  }

  /* Control flow: ternary, match, forall/exists */
  if let Some(ternary) = TernaryExpr::cast(node.clone()) {
    let condition = posttype_evaluate_node(db, scope, &ternary.condition()?)?;
    if let Some(bool) = condition.as_bool(db) {
      if bool {
        return posttype_evaluate_node(db, scope, &ternary.then_expr()?);
      } else {
        return posttype_evaluate_node(db, scope, &ternary.else_expr()?);
      }
    }
    return None;
  }

  if let Some(match_expr) = MatchExpr::cast(node.clone()) {
    let subject = posttype_evaluate_node(db, scope, &match_expr.subject()?)?;
    for arm in match_expr.arms() {
      let pattern = posttype_evaluate_node(db, scope, &arm.pattern()?)?;
      if obj_value_eq(db, &subject, &pattern) {
        return posttype_evaluate_node(db, scope, &arm.result()?);
      }
    }
    return None;
  }

  if let Some(quantifier) = QuantifierExpr::cast(node.clone()) {
    return evaluate_quantifier(db, &quantifier, scope, quantifier.is_forall());
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
    return Some(StringObj::new(db, single_quoted_string.content()).into());
  }
  if let Some(opaque_string) = OqStringExpr::cast(node.clone()) {
    return Some(OStringObj::new(db, opaque_string.content()).into());
  }
  if let Some(double_quoted_string) = DqStringExpr::cast(node.clone()) {
    // Double-quoted strings in type position are referees like `"My Table"`
    if let Some(result) = *posttype_resolve_referee(db, scope.file(db), node)
      && let Some(symbol) = result.symbol
    {
      return *pretype_evaluate_typ_symbol(db, symbol);
    }
    // outside type position: treat as a string literal
    return Some(StringObj::new(db, double_quoted_string.content()).into());
  }

  if let Some(triple_quoted_string) = TqStringExpr::cast(node.clone()) {
    // triple-quoted multiline string: `'''text'''` (dedented and unescaped)
    return Some(StringObj::new(db, triple_quoted_string.value()).into());
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
      .filter_map(|item| posttype_evaluate_node(db, scope, &item))
      .collect();
    return Some(ListObj::new(db, items).into());
  }

  if let Some(tuple) = TupleExpr::cast(node.clone()) {
    // `(a, b, c)` / `()`: tuple literal, evaluates as a list
    let items: Vec<Obj<'db>> = tuple
      .items()
      .filter_map(|item| posttype_evaluate_node(db, scope, &item))
      .collect();
    return Some(ListObj::new(db, items).into());
  }

  if let Some(csv) = CsvExpr::cast(node.clone()) {
    let null: Obj<'db> = NullObj::new(db).into();
    let items: Vec<Obj<'db>> = csv
      .slots()
      .into_iter()
      .map(|slot| {
        slot
          .and_then(|expr| posttype_evaluate_node(db, scope, &expr))
          .unwrap_or(null)
      })
      .collect();
    return Some(ListObj::new(db, items).into());
  }

  if let Some(number) = NumberExpr::cast(node.clone()) {
    if let Some(integer) = number.int_value() {
      return Some(IntegerObj::new(db, integer).into());
    }
    return Some(FloatObj::new(db, number.raw_text()).into());
  }

  /* Element construction */
  if let Some(element_declaration) = ElementDeclaration::cast(node) {
    return evaluate_element(db, scope, &element_declaration).map(Obj::Element);
  }

  None
}

/// Returns a LazyTyp for an expression in posttype context
pub fn posttype_evaluate_lazy_typ<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  expr: &Expr,
) -> Option<LazyTyp<'db>> {
  let node = expr.syntax().clone();
  if IdentExpr::cast(node.clone()).is_some()
    && let Some(result) = *posttype_resolve_referee(db, scope.file(db), node)
    && let Some(symbol) = result.symbol
  {
    return Some(LazyTyp::lazy(symbol));
  }
  posttype_evaluate_node(db, scope, expr)
    .and_then(|obj| obj.as_typ())
    .map(LazyTyp::eager)
}

/// Resolves the field line type by walking up to the parent element declaration
fn resolve_field_line_typ<'db>(
  db: &'db dyn Database,
  file: File,
  field_line: &ElementFieldDeclaration,
) -> Option<ElementTyp<'db>> {
  let parent_body = field_line.syntax().parent()?;
  let parent_element = parent_body.parent()?;
  let Typ::Element(parent_typ) = (*get_declaration_typ(db, file, parent_element))? else {
    return None;
  };
  parent_typ.field_line_typ(db)
}

// Expression evaluation helpers

// Resolves an operator from scope and dispatches execution
// User-defined operators are called via FuncObj, virtual operators via evaluate_builtin_infix
fn dispatch_operator<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  infix: &InfixExpr,
  op: &str,
  left: Obj<'db>,
) -> Option<Obj<'db>> {
  let file = scope.file(db);
  let mut current_scope = Some(*get_node_static_scope(db, file, infix.syntax().clone()));
  while let Some(s) = current_scope {
    let members = posttype_get_scope_members(db, s);
    if let Some(symbol) = members
      .lookup_symbols(db, op, SymbolCategoryKind::Operator)
      .next()
    {
      // Unwrap Use wrappers
      let mut current = symbol;
      while let SymbolKind::Use(inner, _, _, _) = current.kind(db) {
        current = *inner;
      }
      return match current.kind(db) {
        SymbolKind::UserOperator(op_file, node, _) => {
          let func = FuncDeclaration::cast(node.clone())?;
          let any_typ = LazyTyp::eager(ObjTyp::new(db).into());
          let signature = FuncSignature::new(db, vec![any_typ, any_typ], any_typ);
          let func_typ = FuncTyp::new(db, signature);
          let func_obj = FuncObj::new(
            db,
            format!("operator{op}"),
            func_typ,
            Some(func.syntax().clone()),
            Some(RuntimeScope::empty(db, *op_file)),
          );
          let right = posttype_evaluate_node(db, scope, &infix.right()?)?;
          call_func_obj(db, func_obj, vec![left, right], scope)
        }
        SymbolKind::VirtualOperator(_) => {
          let right = posttype_evaluate_node(db, scope, &infix.right()?);
          evaluate_builtin_infix(db, &left, &right, op)
        }
        _ => None,
      };
    }
    current_scope = s.get_parent(db);
  }
  // Every operator should have a symbol in Builtin scope
  None
}

/// Evaluates a forall/exists quantifier over a list - `is_all` true means forall, false means exists
fn evaluate_quantifier<'db>(
  db: &'db dyn Database,
  expr: &QuantifierExpr,
  scope: RuntimeScope<'db>,
  is_all: bool,
) -> Option<Obj<'db>> {
  let binding_name = expr.binding()?;
  let collection = posttype_evaluate_node(db, scope, &expr.collection()?)?;
  let body_expr = expr.body_expr()?;

  let items = match collection {
    Obj::List(list) => list.items(db).clone(),
    _ => return None,
  };

  for item in &items {
    let child_scope = scope.child(db, BTreeMap::from([(binding_name.clone(), *item)]));
    let result = posttype_evaluate_node(db, child_scope, &body_expr);
    let is_true = matches!(result, Some(Obj::Bool(bool)) if bool.value(db));
    if is_all && !is_true {
      return Some(BoolObj::new(db, false).into());
    }
    if !is_all && is_true {
      return Some(BoolObj::new(db, true).into());
    }
  }

  Some(BoolObj::new(db, is_all).into())
}

/// Dispatches infix expression evaluation: dot access, field access, type union, and arithmetic
fn evaluate_infix<'db>(
  db: &'db dyn Database,
  infix: &InfixExpr,
  scope: RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  let op = infix.op().as_deref()?.to_string();

  match op.as_str() {
    "|" => {
      // type union: `A | B` builds a sum type
      let left = posttype_evaluate_node(db, scope, &infix.left()?)
        .and_then(|obj| obj.coerce_to_union_member_typ(db))?;
      let right = posttype_evaluate_node(db, scope, &infix.right()?)
        .and_then(|obj| obj.coerce_to_union_member_typ(db))?;
      return Some(Obj::Typ(SumTyp::create(db, vec![left, right])));
    }
    "@" => {
      // field access: `obj @ field` reads a named field from an element or object
      let left = posttype_evaluate_node(db, scope, &infix.left()?)?;
      let field_name = infix.right()?.as_name()?;
      return Some(
        left
          .lookup_field(db, &field_name)
          .unwrap_or_else(|| NullObj::new(db).into()),
      );
    }
    "." => {
      // Namespace traversal: `a.b` follows static scope then falls back to runtime lookup
      let right = infix.right()?;
      if let Some(result) = *posttype_resolve_referee(db, scope.file(db), right.syntax().clone())
        && let Some(symbol) = result.symbol
      {
        // Resolved to a symbol: evaluate it
        if let SymbolKind::UserElement(element_file, element_node, _) = symbol.kind(db) {
          if let Some(element_declaration) = ElementDeclaration::cast(element_node.clone()) {
            let element_scope = RuntimeScope::empty(db, *element_file);
            return evaluate_element(db, element_scope, &element_declaration).map(Obj::Element);
          }
          // Field line declarations like columns declared via [label]
          if let Some(field_declaration) = ElementFieldDeclaration::cast(element_node.clone())
            && let Some(field_typ) = resolve_field_line_typ(db, *element_file, &field_declaration)
          {
            let element_scope = RuntimeScope::empty(db, *element_file);
            return evaluate_field_line(db, element_scope, &field_declaration, field_typ);
          }
        }
        return *pretype_evaluate_typ_symbol(db, symbol);
      }
      // Fallback: evaluate left and do runtime namespace lookup
      let left = posttype_evaluate_node(db, scope, &infix.left()?)?;
      let member_name = right.as_name()?;
      if let Some(member) = left.lookup_namespace_member(db, &member_name) {
        // Getter FuncObj with 0 declared params: call with element fields as scope
        if let Obj::Func(func_obj) = member
          && func_obj.signature(db).params(db).is_empty()
        {
          let getter_scope = if let Obj::Element(element_obj) = left {
            let bindings: BTreeMap<_, _> = element_obj.fields(db).clone();
            scope.child(db, bindings)
          } else {
            scope
          };
          return call_func_obj(db, func_obj, vec![], getter_scope);
        }
        return Some(member);
      }
      // Namespace lookup missed: emit diagnostics based on the type namespace shape
      if let Obj::Element(element) = left
        && let Typ::Element(element_typ) = element.element_typ(db)
      {
        let namespace = element_typ.typ_namespace(db);
        if namespace.expanded_field.is_some() {
          // Expanded field searched but named item not found at runtime
          let right_node = right.syntax();
          Diagnostics::accumulate(
            Diagnostics(
              DiagnosticKind::MissingExpandedNamespaceMember { name: member_name }
                .into_error(right_node.start_offset(), right_node.end_offset()),
            ),
            db,
          );
          return Some(NullObj::new(db).into());
        }
        if !namespace.members.is_empty() || !namespace.getters.is_empty() {
          // Type has a namespace but member not found: check fields before erroring
          if let Some(field) = left.lookup_field(db, &member_name) {
            return Some(field);
          }
          let right_node = right.syntax();
          Diagnostics::accumulate(
            Diagnostics(
              DiagnosticKind::UnknownNamespaceMember { name: member_name }
                .into_error(right_node.start_offset(), right_node.end_offset()),
            ),
            db,
          );
          return None;
        }
      }
      return left.lookup_field(db, &member_name);
    }
    _ => {}
  }

  let left = posttype_evaluate_node(db, scope, &infix.left()?)?;

  // Type-level method operators (e.g. List.filter) take priority
  if let Some(func_obj) = left.get_typ(db).lookup_runtime_op(db, op.as_str()) {
    let right = posttype_evaluate_node(db, scope, &infix.right()?)?;
    return call_func_obj(db, func_obj, vec![left, right], scope);
  }

  // Operator dispatch: resolve from innermost scope upward, then execute
  dispatch_operator(db, scope, infix, &op, left)
}

/// Evaluates built-in infix operators: `??`, `==`, `!=`, `&&`, `||`, arithmetic, string `++`
fn evaluate_builtin_infix<'db>(
  db: &'db dyn Database,
  left: &Obj<'db>,
  right: &Option<Obj<'db>>,
  op: &str,
) -> Option<Obj<'db>> {
  if op == "??" {
    // `??`: null coalescing, returns left if non-null, else right
    return if !matches!(left, Obj::Null(_)) {
      Some(*left)
    } else {
      *right
    };
  }

  if let Some(right) = right {
    match op {
      // `==` / `!=`: value equality (works across null, bool, int, float, string)
      "==" => return Some(BoolObj::new(db, obj_value_eq(db, left, right)).into()),
      "!=" => return Some(BoolObj::new(db, !obj_value_eq(db, left, right)).into()),
      _ => {}
    }
  }

  let left_bool = left.as_bool(db).unwrap_or(false);
  let right_bool = right.and_then(|rhs| rhs.as_bool(db)).unwrap_or(false);
  match op {
    // `&&` / `||` / `^`: boolean logic
    "&&" => return Some(BoolObj::new(db, left_bool && right_bool).into()),
    "||" => return Some(BoolObj::new(db, left_bool || right_bool).into()),
    "^" => return Some(BoolObj::new(db, left_bool ^ right_bool).into()),
    _ => {}
  }

  // arithmetic and comparison operators on numeric types
  if let Obj::Integer(left_int) = left {
    let left_value = left_int.value(db);
    if let Some(Obj::Integer(right_int)) = right {
      let right_value = right_int.value(db);
      return match op {
        "+" => Some(IntegerObj::new(db, left_value.wrapping_add(right_value)).into()),
        "-" => Some(IntegerObj::new(db, left_value.wrapping_sub(right_value)).into()),
        "*" => Some(IntegerObj::new(db, left_value.wrapping_mul(right_value)).into()),
        "/" if right_value != 0 => Some(IntegerObj::new(db, left_value / right_value).into()),
        "%" if right_value != 0 => Some(IntegerObj::new(db, left_value % right_value).into()),
        "<" => Some(BoolObj::new(db, left_value < right_value).into()),
        ">" => Some(BoolObj::new(db, left_value > right_value).into()),
        "<=" => Some(BoolObj::new(db, left_value <= right_value).into()),
        ">=" => Some(BoolObj::new(db, left_value >= right_value).into()),
        // `..`: integer range, produces a list
        ".." => {
          let items: Vec<Obj<'db>> = (left_value..right_value)
            .map(|item| IntegerObj::new(db, item).into())
            .collect();
          Some(ListObj::new(db, items).into())
        }
        _ => None,
      };
    }
    if let Some(Obj::Float(right_float)) = right {
      let left_float: f64 = left_value as f64;
      let right_float: f64 = right_float.value(db).parse().ok()?;
      return match op {
        "+" => Some(FloatObj::new(db, (left_float + right_float).to_string()).into()),
        "-" => Some(FloatObj::new(db, (left_float - right_float).to_string()).into()),
        "*" => Some(FloatObj::new(db, (left_float * right_float).to_string()).into()),
        "/" if right_float != 0.0 => {
          Some(FloatObj::new(db, (left_float / right_float).to_string()).into())
        }
        "%" if right_float != 0.0 => {
          Some(FloatObj::new(db, (left_float % right_float).to_string()).into())
        }
        "<" => Some(BoolObj::new(db, left_float < right_float).into()),
        ">" => Some(BoolObj::new(db, left_float > right_float).into()),
        "<=" => Some(BoolObj::new(db, left_float <= right_float).into()),
        ">=" => Some(BoolObj::new(db, left_float >= right_float).into()),
        _ => None,
      };
    }
  }

  if let Obj::Float(left_float) = left {
    let left_value: f64 = left_float.value(db).parse().ok()?;
    let right_value: f64 = match right {
      Some(Obj::Float(right_float)) => right_float.value(db).parse().ok()?,
      Some(Obj::Integer(right_int)) => right_int.value(db) as f64,
      _ => return None,
    };
    return match op {
      "+" => Some(FloatObj::new(db, (left_value + right_value).to_string()).into()),
      "-" => Some(FloatObj::new(db, (left_value - right_value).to_string()).into()),
      "*" => Some(FloatObj::new(db, (left_value * right_value).to_string()).into()),
      "/" if right_value != 0.0 => {
        Some(FloatObj::new(db, (left_value / right_value).to_string()).into())
      }
      "%" if right_value != 0.0 => {
        Some(FloatObj::new(db, (left_value % right_value).to_string()).into())
      }
      "<" => Some(BoolObj::new(db, left_value < right_value).into()),
      ">" => Some(BoolObj::new(db, left_value > right_value).into()),
      "<=" => Some(BoolObj::new(db, left_value <= right_value).into()),
      ">=" => Some(BoolObj::new(db, left_value >= right_value).into()),
      _ => None,
    };
  }

  // `++`: string concatenation
  if let (Obj::String(left_string), Some(Obj::String(right_string)), "++") = (left, right, op) {
    return Some(
      StringObj::new(
        db,
        format!("{}{}", left_string.value(db), right_string.value(db)),
      )
      .into(),
    );
  }

  None
}

/// Evaluates a built-in list method call (filter, map, every, some, flatMap, indices, concat)
fn evaluate_list_method<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  list: ListObj<'db>,
  method: &str,
  args: &[Obj<'db>],
) -> Option<Obj<'db>> {
  let items = list.items(db);
  match method {
    "indices" => {
      let range: Vec<Obj<'db>> = (0..items.len() as i64)
        .map(|item| IntegerObj::new(db, item).into())
        .collect();
      Some(ListObj::new(db, range).into())
    }
    "filter" => {
      let Obj::Func(func) = args.first()? else {
        return None;
      };
      let filtered: Vec<Obj<'db>> = items
        .iter()
        .filter(|item| {
          call_func_obj(db, *func, vec![**item], scope)
            .and_then(|obj| obj.as_bool(db))
            .unwrap_or(false)
        })
        .copied()
        .collect();
      Some(ListObj::new(db, filtered).into())
    }
    "map" => {
      let Obj::Func(func) = args.first()? else {
        return None;
      };
      let mapped: Vec<Obj<'db>> = items
        .iter()
        .filter_map(|item| call_func_obj(db, *func, vec![*item], scope))
        .collect();
      Some(ListObj::new(db, mapped).into())
    }
    "every" => {
      let Obj::Func(func) = args.first()? else {
        return None;
      };
      let result = items.iter().all(|item| {
        call_func_obj(db, *func, vec![*item], scope)
          .and_then(|obj| obj.as_bool(db))
          .unwrap_or(false)
      });
      Some(BoolObj::new(db, result).into())
    }
    "some" => {
      let Obj::Func(func) = args.first()? else {
        return None;
      };
      let result = items.iter().any(|item| {
        call_func_obj(db, *func, vec![*item], scope)
          .and_then(|obj| obj.as_bool(db))
          .unwrap_or(false)
      });
      Some(BoolObj::new(db, result).into())
    }
    "flatMap" => {
      let Obj::Func(func) = args.first()? else {
        return None;
      };
      let mut result = Vec::new();
      for item in items {
        if let Some(Obj::List(inner)) = call_func_obj(db, *func, vec![*item], scope) {
          result.extend(inner.items(db).iter().copied());
        }
      }
      Some(ListObj::new(db, result).into())
    }
    "concat" | "++" => {
      let Obj::List(other) = args.first()? else {
        return None;
      };
      let mut result = items.to_vec();
      result.extend(other.items(db).iter().copied());
      Some(ListObj::new(db, result).into())
    }
    _ => None,
  }
}

/// Evaluates a call expression: method call on dot-chain, type construction, or function call
fn evaluate_call<'db>(
  db: &'db dyn Database,
  call: &CallExpr,
  scope: RuntimeScope<'db>,
) -> Option<Obj<'db>> {
  let callee_expr = call.callee()?;

  if let Some(infix) = InfixExpr::cast(callee_expr.syntax().clone())
    && infix.has_op(".")
  {
    let base = posttype_evaluate_node(db, scope, &infix.left()?)?;
    let method_name = infix.right()?.as_name()?;
    let args: Vec<Obj<'db>> = call
      .args()
      .filter_map(|arg| posttype_evaluate_node(db, scope, &arg))
      .collect();

    if let Some(func_obj) = base.get_typ(db).lookup_runtime_op(db, &method_name) {
      let mut all_args = vec![base];
      all_args.extend(args);
      if func_obj.source(db).is_none() {
        return call_virtual_func(db, scope, func_obj, all_args);
      }
      return call_func_obj(db, func_obj, all_args, scope);
    }

    return None;
  }

  let callee = posttype_evaluate_node(db, scope, &callee_expr)?;
  let args: Vec<Obj<'db>> = call
    .args()
    .filter_map(|arg| posttype_evaluate_node(db, scope, &arg))
    .collect();
  if let Some(typ) = callee.as_typ()
    && typ.call_typ(db).is_some()
  {
    return typ
      .construct(db, args.iter().filter_map(|arg| arg.as_typ()).collect())
      .map(Obj::Typ);
  }
  if let Obj::Func(func_obj) = callee {
    if func_obj.source(db).is_none() {
      return call_virtual_func(db, scope, func_obj, args);
    }
    return call_func_obj(db, func_obj, args, scope);
  }
  None
}

/// Calls a virtual (source-less) function by name: interpret, typeof, and list methods
fn call_virtual_func<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  func_obj: FuncObj<'db>,
  args: Vec<Obj<'db>>,
) -> Option<Obj<'db>> {
  match func_obj.name(db).as_str() {
    "interpret" => {
      if let Some(arg_typ) = args.first().and_then(|arg| arg.as_typ()) {
        return Some(Obj::Typ(arg_typ));
      }
      Some(Obj::Typ(ObjTyp::new(db).into()))
    }
    "typeof" => args.first().map(|arg| Obj::Typ(arg.get_typ(db))),
    "filter" | "map" | "every" | "some" | "flatMap" | "indices" | "concat" => {
      let Obj::List(list) = args.first()? else {
        return None;
      };
      evaluate_list_method(db, scope, *list, func_obj.name(db).as_str(), &args[1..])
    }
    _ => None,
  }
}

/// Calls a user-defined function or closure by binding params to args and evaluating the body
fn call_func_obj<'db>(
  db: &'db dyn Database,
  func_obj: FuncObj<'db>,
  args: Vec<Obj<'db>>,
  call_scope: RuntimeScope<'db>,
) -> Option<Obj<'db>> {
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
    return posttype_evaluate_node(db, child_scope, &body);
  }
  if let Some(func_declaration) = FuncDeclaration::cast(source.clone()) {
    let mut bindings = BTreeMap::new();
    if let Some(params_node) = func_declaration.params() {
      for (param, arg) in params_node.params().zip(args) {
        if let Some(ident) = param.name() {
          bindings.insert(ident.name(), arg);
        }
      }
    }
    let child_scope = defining_scope.child(db, bindings);
    return func_declaration
      .body()
      .and_then(|body| evaluate_body_expr(db, child_scope, &body));
  }
  if let Some(get_declaration) = GetDeclaration::cast(source) {
    return get_declaration
      .body()
      .and_then(|body| evaluate_body_expr(db, defining_scope, &body));
  }
  None
}

/// Evaluates the first expression found in a block body
fn evaluate_body_expr<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  body: &BlockElementDeclarationBody,
) -> Option<Obj<'db>> {
  for child in body.syntax().children() {
    if let Some(expr) = Expr::cast(child.clone()) {
      return posttype_evaluate_node(db, scope, &expr);
    }
  }
  None
}

/// Extracts parameter names from a closure expression (single ident or destructured tuple)
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

/// Wraps a closure expression as a FuncObj capturing the current scope
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
  let any_typ = LazyTyp::eager(ObjTyp::new(db).into());
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

/// Returns true if two objects have equal scalar values (null, bool, int, float, string)
fn obj_value_eq<'db>(db: &'db dyn Database, left: &Obj<'db>, right: &Obj<'db>) -> bool {
  match (left, right) {
    (Obj::Null(_), Obj::Null(_)) => true,
    (Obj::Bool(lhs), Obj::Bool(rhs)) => lhs.value(db) == rhs.value(db),
    (Obj::Integer(lhs), Obj::Integer(rhs)) => lhs.value(db) == rhs.value(db),
    (Obj::Float(lhs), Obj::Float(rhs)) => lhs.value(db) == rhs.value(db),
    (Obj::String(lhs), Obj::String(rhs)) => lhs.value(db) == rhs.value(db),
    _ => false,
  }
}

// Element evaluation helpers

/// Evaluates an expression into a slot: declaration-typed and [text] slots capture as plain strings
fn evaluate_in_slot<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  expr: &Expr,
  arg_definition: &ArgFieldDefinition<'db>,
) -> Option<Obj<'db>> {
  let expected_typ = arg_definition.typ.resolve(db);
  if is_declaration_slot(expected_typ) {
    if IdentExpr::cast(expr.syntax().clone()).is_some() {
      return posttype_evaluate_node(db, scope, expr)
        .or_else(|| extract_declaration_name(expr).map(|name| StringObj::new(db, name).into()));
    }
    return extract_declaration_name(expr).map(|name| StringObj::new(db, name).into());
  }
  if arg_definition.is_text {
    // [text] fields: try name extraction first, fall back to raw node text
    return extract_declaration_name(expr)
      .or_else(|| {
        let text = expr.syntax().text().to_string();
        let trimmed = text.trim();
        if trimmed.is_empty() {
          None
        } else {
          Some(trimmed.to_string())
        }
      })
      .map(|name| StringObj::new(db, name).into());
  }
  posttype_evaluate_node(db, scope, expr)
}

/// Evaluates a setting item: bare flag becomes `true`, keyed value evaluates its expression
fn evaluate_setting_value<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  item: &SettingListItem,
) -> Option<Obj<'db>> {
  if let Some(expr) = item.value().and_then(|value| value.expr()) {
    posttype_evaluate_node(db, scope, &expr)
      .or_else(|| extract_declaration_name(&expr).map(|name| StringObj::new(db, name).into()))
  } else {
    Some(BoolObj::new(db, true).into())
  }
}

/// Evaluates a single field line into an ElementObj
fn evaluate_field_line<'db>(
  db: &'db dyn Database,
  scope: RuntimeScope<'db>,
  field_line: &ElementFieldDeclaration,
  resolved_field_typ: ElementTyp<'db>,
) -> Option<Obj<'db>> {
  let args: Vec<_> = field_line.args().collect();
  let mut line_fields = BTreeMap::new();
  let arg_fields = resolved_field_typ.typ_element_arg_fields(db);

  // When the single arg evaluates to an ElementObj matching this field type,
  // use it as the field line and merge settings
  if args.len() == 1
    && let Some(expr) = args[0].expr()
    && let Some(Obj::Element(result)) = posttype_evaluate_node(db, scope, &expr)
    && matches!(result.element_typ(db), Typ::Element(element) if element.name(db) == resolved_field_typ.name(db))
  {
    let mut merged = result.fields(db).clone();
    if let Some(setting_list) = field_line.setting_list() {
      for item in setting_list.items() {
        let Some(name_node) = item.name() else {
          continue;
        };
        let setting_key = name_node.name();
        let field_name = resolved_field_typ.setting_field_name(db, &setting_key);
        if let Some(obj) = evaluate_setting_value(db, scope, &item) {
          merged.insert(field_name, obj);
        }
      }
    }
    return Some(Obj::Element(ElementObj::new(
      db,
      Typ::Element(resolved_field_typ),
      merged,
    )));
  }

  for (index, arg_definition) in arg_fields.iter() {
    if arg_definition.is_variadic {
      let remaining: Vec<Obj<'db>> = args[*index..]
        .iter()
        .filter_map(|arg| {
          arg
            .expr()
            .and_then(|expr| evaluate_in_slot(db, scope, &expr, arg_definition))
        })
        .collect();
      if !remaining.is_empty() {
        line_fields.insert(
          arg_definition.name.clone(),
          ListObj::new(db, remaining).into(),
        );
      }
      break;
    }
    if let Some(arg) = args.get(*index)
      && let Some(expr) = arg.expr()
      && let Some(obj) = evaluate_in_slot(db, scope, &expr, arg_definition)
    {
      let stored = if arg_definition.has_csv && !matches!(obj, Obj::List(_)) {
        ListObj::new(db, vec![obj]).into()
      } else {
        obj
      };
      line_fields.insert(arg_definition.name.clone(), stored);
    }
  }

  if let Some(setting_list) = field_line.setting_list() {
    for item in setting_list.items() {
      let Some(name_node) = item.name() else {
        continue;
      };
      let setting_key = name_node.name();
      let field_name = resolved_field_typ.setting_field_name(db, &setting_key);
      if let Some(obj) = evaluate_setting_value(db, scope, &item) {
        line_fields.insert(field_name, obj);
      }
    }
  }

  Some(Obj::Element(ElementObj::new(
    db,
    Typ::Element(resolved_field_typ),
    line_fields,
  )))
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use crate::Obj;
  use crate::db::DboxideDatabase;
  use crate::db::test_utils::*;
  use crate::db::types::{Diagnostics, RuntimeObj};
  use crate::diagnostics::DiagnosticKind;

  use super::evaluate_file;

  mod namespace_expanded {
    use itertools::Either;

    use crate::db::types::interned::symbol::VirtualModuleKind;

    use super::*;

    fn metatype() -> HashMap<String, Either<VirtualModuleKind, crate::db::types::File>> {
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )])
    }

    // Inline colon-form elements place the field arg in UserModule scope (file-level accessible)

    #[test]
    fn expanded_item_found_no_diagnostic() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.td",
        r#"use * from 'metatype'
type Table [element] {
  name string [label]
  type Col [field] { name string [label, arg: 0] }
  cols Col[]
  namespace { cols [expand] }
}
type Probe [element] {
  name string [label]
  type Arg [field] { target string [arg: 0] }
  args Arg[]
}
Table t { col_a }
Probe p: t.col_a
"#,
        metatype(),
      );
      evaluate_file(&db, file);
      let diagnostics = evaluate_file::accumulated::<Diagnostics>(&db, file);
      assert!(
        !diagnostics.iter().any(|diagnostic| matches!(
          &diagnostic.0.kind,
          DiagnosticKind::MissingExpandedNamespaceMember { .. }
        )),
        "should not emit MissingExpandedNamespaceMember when item exists"
      );
    }

    #[test]
    fn missing_expanded_item_emits_diagnostic() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.td",
        r#"use * from 'metatype'
type Table [element] {
  name string [label]
  type Col [field] { name string [label, arg: 0] }
  cols Col[]
  namespace { cols [expand] }
}
type Probe [element] {
  name string [label]
  type Arg [field] { target string [arg: 0] }
  args Arg[]
}
Table t { col_a }
Probe p: t.nonexistent
"#,
        metatype(),
      );
      evaluate_file(&db, file);
      let diagnostics = evaluate_file::accumulated::<Diagnostics>(&db, file);
      assert!(
        diagnostics.iter().any(|diagnostic| matches!(
          &diagnostic.0.kind,
          DiagnosticKind::MissingExpandedNamespaceMember { name } if name == "nonexistent"
        )),
        "expected MissingExpandedNamespaceMember diagnostic"
      );
    }
  }

  mod namespace_unknown_member {
    use itertools::Either;

    use crate::db::types::interned::symbol::VirtualModuleKind;

    use super::*;

    fn metatype() -> HashMap<String, Either<VirtualModuleKind, crate::db::types::File>> {
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )])
    }

    // Widget re-exports `width` in its namespace and has a nested [field] for inline syntax

    #[test]
    fn unknown_member_emits_diagnostic() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.td",
        r#"use * from 'metatype'
type Widget [element] {
  name string [label]
  width int
  namespace { width }
  type Probe [field] { target string [arg: 0] }
  probes Probe[]
}
Widget foo {}
Widget bar: foo.unknown
"#,
        metatype(),
      );
      evaluate_file(&db, file);
      let diagnostics = evaluate_file::accumulated::<Diagnostics>(&db, file);
      assert!(
        diagnostics.iter().any(|diagnostic| matches!(
          &diagnostic.0.kind,
          DiagnosticKind::UnknownNamespaceMember { name } if name == "unknown"
        )),
        "expected UnknownNamespaceMember diagnostic"
      );
    }

    #[test]
    fn field_value_found_via_lookup_no_diagnostic() {
      let db = DboxideDatabase::default();
      // `foo` has `width: 42` attribute so foo.width is found via lookup_field
      let file = create_file(
        &db,
        "test.td",
        r#"use * from 'metatype'
type Widget [element] {
  name string [label]
  width int
  namespace { width }
  type Probe [field] { target string [arg: 0] }
  probes Probe[]
}
Widget foo {
  width: 42
}
Widget bar: foo.width
"#,
        metatype(),
      );
      evaluate_file(&db, file);
      let diagnostics = evaluate_file::accumulated::<Diagnostics>(&db, file);
      assert!(
        !diagnostics.iter().any(|diagnostic| matches!(
          &diagnostic.0.kind,
          DiagnosticKind::UnknownNamespaceMember { .. }
        )),
        "should not emit UnknownNamespaceMember when field value is found"
      );
    }
  }

  mod namespace_getter {
    use super::*;

    #[test]
    fn getter_on_element_instance_returns_func_obj() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.td",
        r#"type Item {
  name string [label]
  namespace {
    get greeting(): string {
      'hello'
    }
  }
}
Item foo {}"#,
        HashMap::new(),
      );
      let elements = evaluate_file(&db, file);
      assert_eq!(elements.len(), 1);
      let element = elements[0];
      let result = element.lookup_namespace_member(&db, "greeting");
      assert!(result.is_some(), "getter should resolve");
      assert!(
        matches!(result.unwrap(), Obj::Func(_)),
        "getter should return Obj::Func"
      );
    }

    #[test]
    fn nonexistent_getter_returns_none() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.td",
        r#"type Item {
  name string [label]
  namespace {
    get label(): string {
      name
    }
  }
}
Item foo {}"#,
        HashMap::new(),
      );
      let elements = evaluate_file(&db, file);
      assert_eq!(elements.len(), 1);
      let element = elements[0];
      assert!(
        element
          .lookup_namespace_member(&db, "nonexistent")
          .is_none(),
        "missing getter should return None"
      );
    }
  }
}
