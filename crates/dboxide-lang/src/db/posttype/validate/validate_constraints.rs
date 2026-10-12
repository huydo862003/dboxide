use std::collections::BTreeMap;

use salsa::Database;

use crate::ast::{AstNode, ClosureExpr};
use crate::db::types::{Diagnostics, ElementTyp, File, Obj};
use crate::diagnostics::DiagnosticKind;

use crate::db::prepost::RuntimeScope;
use crate::db::prepost::evaluate_node::evaluate_node_posttype;

// Validate constraints on an element instance
// Each constraint is a named closure that must evaluate to true
// Field names in the closure body resolve to field values on the element instance
pub fn validate_constraints<'db>(
  db: &'db dyn Database,
  file: File,
  element_typ: ElementTyp<'db>,
  field_values: &BTreeMap<String, Obj<'db>>,
) {
  let scope = RuntimeScope::with_bindings(db, field_values.clone());

  for constraint in element_typ.constraint_definitions(db) {
    let closure_node = &constraint.closure_syntax;
    let Some(closure) = ClosureExpr::cast(closure_node.clone()) else {
      continue;
    };
    let Some(body) = closure.body() else {
      continue;
    };

    let result = evaluate_node_posttype(db, file, &body, scope);

    let is_valid = match result {
      Some(Obj::Bool(b)) => b.value(db),
      Some(_) => true,
      None => true, // if evaluation fails, skip constraint
    };

    if !is_valid {
      let start = closure_node.offset();
      salsa::Accumulator::accumulate(
        Diagnostics(
          DiagnosticKind::ConstraintViolation {
            constraint_name: constraint.name.clone(),
            typ_name: element_typ.name(db).clone(),
          }
          .into_error(start, start + closure_node.text_len()),
        ),
        db,
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use itertools::Either;

  use crate::db::DboxideDatabase;
  use crate::db::prepost::get_scope_members::get_scope_members_posttype;
  use crate::db::test_utils::*;
  use crate::db::types::interned::symbol::{SymbolKind, VirtualModuleKind};
  use crate::db::types::{StaticScope, StaticScopeKind};

  #[test]
  fn validate_constraints_with_false_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Column [field] {
  null bool? [setting]
  pk bool? [setting]

  constraints {
    pk_not_null: () => !pk
  }
}
"#,
      HashMap::from([(
        "metatype".to_string(),
        Either::Left(VirtualModuleKind::Metatype),
      )]),
    );

    let scope = StaticScope::new(&db, StaticScopeKind::UserModule(file));
    let members = get_scope_members_posttype(&db, scope).clone();
    let symbol = get_first_symbol(&members, "Column");
    assert!(matches!(symbol.kind(&db), SymbolKind::UserTyp(..)));
  }
}
