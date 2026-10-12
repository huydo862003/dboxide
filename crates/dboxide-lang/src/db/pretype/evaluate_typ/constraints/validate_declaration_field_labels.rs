use salsa::{Accumulator, Database};

use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::{SymbolKind, VirtualTypKind};
use crate::db::types::{Diagnostics, Typ};
use crate::diagnostics::DiagnosticKind;

use super::super::types::EvaluatedTypField;

/// Validate that a declaration field carries [label] or [label alias]
pub fn validate_declaration_field_labels(
  db: &dyn Database,
  name: &str,
  field: &EvaluatedTypField<'_>,
) {
  let is_declaration = does_field_have_declaration_typ(db, field.typ);
  let (start, end) = field.span();

  if is_declaration && field.label.is_none() {
    let diag = DiagnosticKind::InvalidDeclarationFieldLabel {
      field_name: name.to_string(),
      message: format!(
        "declaration field '{}' must have [label] or [label alias]",
        name
      ),
    }
    .into_error(start, end);
    Accumulator::accumulate(Diagnostics(diag), db);
  }
}

fn does_field_have_declaration_typ<'db>(db: &'db dyn Database, field_typ: LazyTyp<'db>) -> bool {
  match field_typ {
    LazyTyp::Eager(Typ::Declaration(_) | Typ::QualifiedDeclaration(_)) => true,
    LazyTyp::Lazy(symbol) => matches!(
      symbol.kind(db),
      SymbolKind::VirtualTyp(
        _,
        VirtualTypKind::MetatypeDeclaration | VirtualTypKind::MetatypeQualifiedDeclaration
      )
    ),
    _ => false,
  }
}
