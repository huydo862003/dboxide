use salsa::Database;

use crate::db::types::Diagnostics;
use crate::diagnostics::Diagnostic;

use super::super::types::{ArgPosition, EvaluatedFieldDeclaration};

/// Validate that positional [arg: N] indices increment contiguously starting from 0
/// Fields are sorted by index before validation so declaration order doesn't matter
/// Stops checking after a variadic `[arg: n..]` since no more indexed fields are expected
pub fn validate_contiguous_arg_index<'a>(
  db: &dyn Database,
  fields: impl IntoIterator<Item = (&'a str, &'a EvaluatedFieldDeclaration<'a>)>,
) {
  let mut arg_fields: Vec<_> = fields
    .into_iter()
    .filter(|(_, f)| f.arg_position.is_some())
    .collect();
  arg_fields.sort_by_key(|(_, f)| f.arg_position.as_ref().map_or(usize::MAX, |p| p.index()));

  let mut expected = 0usize;
  for (name, field) in arg_fields {
    let arg_position = field.arg_position.as_ref().unwrap();
    let found = arg_position.index();
    if found != expected {
      let (start, end) = field.span();
      let diag = Diagnostic::NonContiguousFieldArgIndex {
        field_name: name.to_string(),
        found_index: found,
        expected_index: expected,
        start_offset: start,
        end_offset: end,
      };
      salsa::Accumulator::accumulate(Diagnostics(diag), db);
    }
    if matches!(arg_position, ArgPosition::Variadic(_)) {
      // variadic captures all remaining args - no further indexed fields expected
      break;
    }
    // advance expected even on error so subsequent fields are validated against their own declared index
    expected = found + 1;
  }
}
