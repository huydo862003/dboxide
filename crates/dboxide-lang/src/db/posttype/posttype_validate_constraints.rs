use std::collections::BTreeMap;

use salsa::Database;

use crate::ast::{AstNode, ClosureExpr};
use crate::db::types::{Diagnostics, ElementTyp, File, Obj};
use crate::diagnostics::Diagnostic;

use super::posttype_evaluate_node::evaluate_with_scope;
use super::runtime_scope::RuntimeScope;

// validate constraints on an element instance
// each constraint is a named closure that must evaluate to true
// field names in the closure body resolve to field values on the element instance
pub fn validate_constraints<'db>(
  db: &'db dyn Database,
  file: File,
  element_typ: ElementTyp<'db>,
  field_values: &BTreeMap<String, Obj<'db>>,
) {
  let scope = RuntimeScope::with_bindings(field_values.clone());

  for constraint in element_typ.constraint_definitions(db) {
    let closure_node = &constraint.closure_syntax;
    let Some(closure) = ClosureExpr::cast(closure_node.clone()) else {
      continue;
    };
    let Some(body) = closure.body() else {
      continue;
    };

    let result = evaluate_with_scope(db, file, &body, &scope);

    let is_valid = match result {
      Some(Obj::Bool(b)) => b.value(db),
      Some(_) => true,
      None => true, // if evaluation fails, skip constraint
    };

    if !is_valid {
      let start = closure_node.offset();
      salsa::Accumulator::accumulate(
        Diagnostics(Diagnostic::ConstraintViolation {
          constraint_name: constraint.name.clone(),
          typ_name: element_typ.name(db).clone(),
          start_offset: start,
          end_offset: start + closure_node.text_len(),
        }),
        db,
      );
    }
  }
}
