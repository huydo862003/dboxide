use salsa::{Accumulator, Database};

use crate::RuntimeScope;
use crate::ast::{AstNode, ClosureExpr};
use crate::db::posttype_evaluate_element::posttype_evaluate_node;
use crate::db::types::{Diagnostics, ElementObj, ElementTyp, File, Obj, Typ};
use crate::diagnostics::DiagnosticKind;

/// Validates constraints on an interpreted element instance
pub fn validate_element_constraints<'db>(
  db: &'db dyn Database,
  file: File,
  element_obj: ElementObj<'db>,
  enclosing_scope: Option<RuntimeScope<'db>>,
) {
  let Typ::Element(element_typ) = element_obj.element_typ(db) else {
    return;
  };
  let fields = element_obj.fields(db).clone();
  let scope = match enclosing_scope {
    Some(parent) => parent.child(db, fields),
    None => RuntimeScope::with_bindings(db, file, fields),
  };

  evaluate_constraints(db, element_typ, scope);

  /* Recursively validate nested elements and field lines */
  let this_scope = Some(scope);
  for value in element_obj.fields(db).values() {
    if let Obj::List(list) = value {
      for item in list.items(db) {
        if let Obj::Element(child) = item {
          let Typ::Element(child_typ) = child.element_typ(db) else {
            continue;
          };
          // Sub-element: validate with current element as enclosing scope
          if child_typ.typ_settings(db).is_element_typ {
            validate_element_constraints(db, file, *child, this_scope);
          }
          // Field line: validate constraints with current element as enclosing scope
          if child_typ.typ_settings(db).is_field_typ {
            let child_fields = child.fields(db).clone();
            let child_scope = scope.child(db, child_fields);
            evaluate_constraints(db, child_typ, child_scope);
          }
        }
      }
    }
  }
}

/// Evaluates all constraint closures on a type against a scope
fn evaluate_constraints<'db>(
  db: &'db dyn Database,
  element_typ: ElementTyp<'db>,
  scope: RuntimeScope<'db>,
) {
  for constraint in element_typ.constraint_definitions(db) {
    let closure_node = &constraint.closure_syntax;
    let Some(closure) = ClosureExpr::cast(closure_node.clone()) else {
      continue;
    };
    let Some(body) = closure.body() else {
      continue;
    };

    let result = posttype_evaluate_node(db, scope, &body);

    let is_valid = match result {
      Some(Obj::Bool(bool)) => bool.value(db),
      Some(_) => true,
      None => true,
    };

    if !is_valid {
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::ConstraintViolation {
            constraint_name: constraint.name.clone(),
            typ_name: element_typ.name(db).clone(),
          }
          .into_error(closure_node.start_offset(), closure_node.end_offset()),
        ),
        db,
      );
    }
  }
}
