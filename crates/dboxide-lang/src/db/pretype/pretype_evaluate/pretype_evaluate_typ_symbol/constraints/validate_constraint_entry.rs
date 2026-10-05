use salsa::Database;

use crate::ast::{AstNode, ClosureExpr, ElementAttributeDeclaration};
use crate::db::types::{ConstraintDefinition, Diagnostics};
use crate::diagnostics::Diagnostic;

use super::super::types::EvaluatedConstraintEntry;

/// Extract a constraint entry from an attribute declaration without validation
/// Returns `None` only if a name cannot be extracted at all
pub fn extract_constraint_entry(
  attr: &ElementAttributeDeclaration,
) -> Option<EvaluatedConstraintEntry> {
  let name_node = attr.name().and_then(|n| n.name())?;
  let name = name_node.as_name()?;
  let closure_syntax = attr
    .value()
    .and_then(|v| v.expr())
    .map(|e| e.unwrap_parens())
    .and_then(|e| ClosureExpr::cast(e.syntax().clone()))
    .map(|c| c.syntax().clone());

  Some(EvaluatedConstraintEntry {
    name,
    closure_syntax,
    node: attr.syntax().clone(),
  })
}

/// Validate a constraint entry: name must be non-empty and value must be a closure
/// Returns the `ConstraintDefinition` if valid, `None` otherwise
pub fn validate_constraint_entry(
  db: &dyn Database,
  entry: &EvaluatedConstraintEntry,
) -> (Vec<Diagnostic>, Option<ConstraintDefinition>) {
  let mut diagnostics = Vec::new();
  let (start, end) = entry.span();

  if entry.name.trim().is_empty() {
    let diag = Diagnostic::InvalidConstraintDefinition {
      constraint_name: entry.name.clone(),
      message: "constraint name cannot be empty".to_string(),
      start_offset: start,
      end_offset: end,
    };
    salsa::Accumulator::accumulate(Diagnostics(diag.clone()), db);
    diagnostics.push(diag);
    return (diagnostics, None);
  }

  match &entry.closure_syntax {
    Some(closure) => (
      diagnostics,
      Some(ConstraintDefinition {
        name: entry.name.clone(),
        closure_syntax: closure.clone(),
      }),
    ),
    None => {
      let diag = Diagnostic::InvalidConstraintDefinition {
        constraint_name: entry.name.clone(),
        message: format!(
          "constraint '{}' value must be a closure expression",
          entry.name
        ),
        start_offset: start,
        end_offset: end,
      };
      salsa::Accumulator::accumulate(Diagnostics(diag.clone()), db);
      diagnostics.push(diag);
      (diagnostics, None)
    }
  }
}
