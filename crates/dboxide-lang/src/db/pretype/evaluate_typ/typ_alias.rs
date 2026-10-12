use salsa::Database;

use crate::ast::EqualityDeclaration;
use crate::db::types::{File, Typ};

use crate::db::prepost::evaluate_node::evaluate_node_pretype;

/// Evaluate `type Name = expr` by resolving the RHS
pub fn evaluate_typ_alias<'db>(
  db: &'db dyn Database,
  file: File,
  equality_declaration: &EqualityDeclaration,
) -> Option<Typ<'db>> {
  let rhs = equality_declaration.rhs()?;
  evaluate_node_pretype(db, file, &rhs)?.as_typ()
}
