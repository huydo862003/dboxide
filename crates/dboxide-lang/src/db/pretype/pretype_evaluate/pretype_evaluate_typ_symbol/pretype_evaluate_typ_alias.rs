use salsa::Database;

use crate::ast::EqualityDeclaration;
use crate::db::types::{File, Typ};

use super::super::pretype_evaluate_node::evaluate_node;

/// Evaluate `type Name = expr` by resolving the RHS
pub fn evaluate_typ_alias<'db>(
  db: &'db dyn Database,
  file: File,
  equality_declaration: &EqualityDeclaration,
) -> Option<Typ<'db>> {
  let rhs = equality_declaration.rhs()?;
  evaluate_node(db, file, &rhs)?.as_typ()
}
