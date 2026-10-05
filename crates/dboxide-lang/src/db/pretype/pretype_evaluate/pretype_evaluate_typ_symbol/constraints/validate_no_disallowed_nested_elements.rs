use salsa::{Accumulator, Database};

use crate::db::types::Diagnostics;
use crate::diagnostics::Diagnostic;

use super::super::types::EvaluatedNestedElement;

/// Validate that no non-type block elements appear directly inside a type body
pub fn validate_no_disallowed_nested_elements(
  db: &dyn Database,
  elements: &[EvaluatedNestedElement],
) {
  for element in elements {
    let (start, end) = element.span();
    Accumulator::accumulate(
      Diagnostics(Diagnostic::DisallowedNestedElement {
        element_typ: element.element_typ.clone(),
        start_offset: start,
        end_offset: end,
      }),
      db,
    );
  }
}
