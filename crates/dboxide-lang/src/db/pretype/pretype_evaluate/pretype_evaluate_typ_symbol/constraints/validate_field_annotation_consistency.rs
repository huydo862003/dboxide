use salsa::{Accumulator, Database};

use crate::db::types::Diagnostics;
use crate::diagnostics::Diagnostic;

use super::super::types::{EvaluatedFieldDeclaration, FieldAnnotationFlags, LabelFlags};

/// Validate annotation consistency rules for a single field declaration:
///
/// - `[label]` and `[label alias]` are mutually exclusive on the same field
/// - `[unique]` requires `[label]` or `[label alias]`
/// - `[csv]` requires `[arg: n]` or `[arg: n..]`
pub fn validate_field_annotation_consistency(
  db: &dyn Database,
  name: &str,
  field: &EvaluatedFieldDeclaration<'_>,
  raw_label: LabelFlags,
) {
  let (start, end) = field.span();

  if raw_label.contains(LabelFlags::LABEL) && raw_label.contains(LabelFlags::ALIAS) {
    Accumulator::accumulate(
      Diagnostics(Diagnostic::ConflictingLabelAnnotations {
        field_name: name.to_string(),
        start_offset: start,
        end_offset: end,
      }),
      db,
    );
  }

  if raw_label.contains(LabelFlags::UNIQUE)
    && !raw_label.intersects(LabelFlags::LABEL | LabelFlags::ALIAS)
  {
    Accumulator::accumulate(
      Diagnostics(Diagnostic::UniqueRequiresLabel {
        field_name: name.to_string(),
        start_offset: start,
        end_offset: end,
      }),
      db,
    );
  }

  if field.flags.contains(FieldAnnotationFlags::CSV) && field.arg_position.is_none() {
    Accumulator::accumulate(
      Diagnostics(Diagnostic::CsvRequiresArg {
        field_name: name.to_string(),
        start_offset: start,
        end_offset: end,
      }),
      db,
    );
  }
}
