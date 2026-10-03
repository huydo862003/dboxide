use salsa::{Accumulator, Database};

use crate::db::types::Diagnostics;
use crate::diagnostics::DiagnosticKind;

use super::super::types::EvaluatedTypField;

/// Validate annotation consistency rules for a single field declaration:
///
/// - `[label]` and `[label alias]` are mutually exclusive on the same field
/// - `[unique]` requires `[label]`, `[label alias]`, or `[setting]` (`[attribute]` is inherently unique)
/// - `[csv]` requires `[arg: n]` or `[arg: n..]`
/// - `[label]`/`[label alias]` conflicts with `[setting]` and `[attribute]`
/// - `[setting]` conflicts with `[arg]`
pub fn validate_field_annotation_consistency(
  db: &dyn Database,
  name: &str,
  field: &EvaluatedTypField<'_>,
) {
  let (start, end) = field.span();

  if field.is_label() && field.is_label_alias() {
    Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::ConflictingLabelAnnotations {
          field_name: name.to_string(),
        }
        .into_error(start, end),
      ),
      db,
    );
  }

  if field.is_unique() && !field.is_label_or_alias() && !field.is_setting() {
    Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::UniqueRequiresLabel {
          field_name: name.to_string(),
        }
        .into_error(start, end),
      ),
      db,
    );
  }

  if field.has_csv() && !field.is_arg() {
    Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::CsvRequiresArg {
          field_name: name.to_string(),
        }
        .into_error(start, end),
      ),
      db,
    );
  }

  let is_label = field.is_label_or_alias();
  let has_setting = field.is_setting();
  let has_attribute = field.is_attribute();
  let has_arg = field.is_arg();

  let conflict = |first: &'static str, second: &'static str| {
    Accumulator::accumulate(
      Diagnostics(
        DiagnosticKind::ConflictingFieldAnnotations {
          field_name: name.to_string(),
          first: first.to_string(),
          second: second.to_string(),
        }
        .into_error(start, end),
      ),
      db,
    );
  };

  // [label] can coexist with [arg] (e.g. `name declaration[T] [arg: 0, label]`)
  if is_label && has_setting {
    conflict("label", "setting");
  }
  if is_label && has_attribute {
    conflict("label", "attribute");
  }
  if has_setting && has_arg {
    conflict("setting", "arg");
  }
}
