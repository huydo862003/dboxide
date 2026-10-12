use salsa::{Accumulator, Database};

use crate::ast::{AstNode, BlockElementDeclaration, ElementDeclaration};
use crate::db::types::Diagnostics;
use crate::diagnostics::DiagnosticKind;

pub fn validate_no_disallowed_nested_elements(
  db: &dyn Database,
  elements: &[BlockElementDeclaration],
) {
  for element in elements {
    let element_typ = ElementDeclaration::cast(element.syntax().clone())
      .and_then(|e| e.typ())
      .map(|t| t.text().trim().to_string())
      .unwrap_or_else(|| "<unknown>".to_string());
    let start = element.syntax().offset();
    Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::DisallowedNestedElement { element_typ }
          .into_error(start, start + element.syntax().text_len()),
      ),
      db,
    );
  }
}
