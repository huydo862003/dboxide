use salsa::{Accumulator, Database};

use crate::db::types::Diagnostics;
use crate::diagnostics::DiagnosticKind;

use super::super::types::EvaluatedTypField;

/// Validate that a type with `[label alias]` fields also has at least one `[label]` field
/// An alias label without a primary label is meaningless, the alias would have nothing to refer to
pub fn validate_alias_has_primary_label<'a>(
  db: &dyn Database,
  fields: impl IntoIterator<Item = (&'a str, &'a EvaluatedTypField<'a>)>,
) {
  let fields: Vec<_> = fields.into_iter().collect();

  let has_primary = fields.iter().any(|(_, field)| field.is_label());

  if has_primary {
    return;
  }

  for (name, field) in &fields {
    if field.is_label_alias() {
      let (start, end) = field.span();
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::AliasLabelWithoutPrimaryLabel {
            field_name: name.to_string(),
          }
          .into_error(start, end),
        ),
        db,
      );
    }
  }
}
