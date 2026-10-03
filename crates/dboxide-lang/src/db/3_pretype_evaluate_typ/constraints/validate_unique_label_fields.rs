use salsa::{Accumulator, Database};

use crate::db::types::Diagnostics;
use crate::diagnostics::DiagnosticKind;

use super::super::typ_element::types::EvaluatedTypField;

/// Validate that a type has at most one [label] field and at most one [label alias] field
pub fn validate_unique_label_fields<'a>(
  db: &dyn Database,
  fields: impl Iterator<Item = (&'a str, &'a EvaluatedTypField<'a>)>,
) {
  let mut first_label: Option<&str> = None;
  let mut first_alias: Option<&str> = None;

  for (name, field) in fields {
    if field.is_label() {
      if let Some(first_label) = first_label {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::DuplicateLabelField {
              first: first_label.to_string(),
              second: name.to_string(),
            }
            .into_error(field.node.start_offset(), field.node.end_offset()),
          ),
          db,
        );
      } else {
        first_label = Some(name);
      }
    }
    if field.is_label_alias() {
      if let Some(first_alias) = first_alias {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::DuplicateLabelAliasField {
              first: first_alias.to_string(),
              second: name.to_string(),
            }
            .into_error(field.node.start_offset(), field.node.end_offset()),
          ),
          db,
        );
      } else {
        first_alias = Some(name);
      }
    }
  }
}
