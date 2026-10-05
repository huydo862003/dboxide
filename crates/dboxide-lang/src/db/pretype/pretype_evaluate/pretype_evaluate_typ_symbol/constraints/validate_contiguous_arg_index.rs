use salsa::Database;

use crate::db::types::Diagnostics;
use crate::diagnostics::Diagnostic;

use super::super::types::{ArgPosition, EvaluatedFieldDeclaration};

pub fn validate_contiguous_arg_index<'a>(
  db: &dyn Database,
  fields: impl IntoIterator<Item = (&'a str, &'a EvaluatedFieldDeclaration<'a>)>,
) {
  // collect only fields with [arg: n] or [arg: n..], sorted by index
  let mut arg_fields: Vec<_> = fields
    .into_iter()
    .filter(|(_, f)| f.arg_position.is_some())
    .collect();
  arg_fields.sort_by_key(|(_, f)| f.arg_position.as_ref().map_or(usize::MAX, |p| p.index()));

  // walk sorted fields
  // check index is contiguous, track variadic to catch anything after it
  let mut expected = 0usize;
  let mut seen_variadic = false;
  for (name, field) in arg_fields {
    let arg_position = field.arg_position.as_ref().unwrap();
    let (start, end) = field.span();

    // [arg: n..] captures all remaining positions - anything after it is unreachable
    if seen_variadic {
      salsa::Accumulator::accumulate(
        Diagnostics(Diagnostic::ArgAfterVariadicArg {
          field_name: name.to_string(),
          start_offset: start,
          end_offset: end,
        }),
        db,
      );
      continue;
    }

    let found = arg_position.index();
    if found != expected {
      salsa::Accumulator::accumulate(
        Diagnostics(Diagnostic::NonContiguousFieldArgIndex {
          field_name: name.to_string(),
          found_index: found,
          expected_index: expected,
          start_offset: start,
          end_offset: end,
        }),
        db,
      );
    }

    if matches!(arg_position, ArgPosition::Variadic(_)) {
      seen_variadic = true;
    } else {
      expected = found + 1;
    }
  }
}
