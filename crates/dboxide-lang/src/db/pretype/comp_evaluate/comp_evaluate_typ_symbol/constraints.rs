use salsa::Database;

use crate::ast::{AstNode, ElementFieldDeclaration};
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::{SymbolKind, VirtualTypKind};
use crate::db::types::{Diagnostics, Typ};
use crate::diagnostics::Diagnostic;

/// Check if a lazy or eager type corresponds to `declaration[T]` or `qualified_declaration[T]`
pub fn is_declaration_typ<'db>(db: &'db dyn Database, field_typ: LazyTyp<'db>) -> bool {
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

/// Validate declaration field label and alias requirements
/// Emits diagnostic if declaration field lacks label or if qualified alias is used
pub fn validate_declaration_field_labels<'db>(
  db: &'db dyn Database,
  field: &ElementFieldDeclaration,
  field_name: &str,
  field_typ: LazyTyp<'db>,
  is_label: bool,
  has_qualified_alias: bool,
) {
  let is_decl = is_declaration_typ(db, field_typ);

  if is_decl && !is_label {
    let (start, len) = (field.syntax().offset(), field.syntax().text_len());
    salsa::Accumulator::accumulate(
      Diagnostics(Diagnostic::InvalidDeclarationFieldLabel {
        field_name: field_name.to_string(),
        message: format!("declaration field '{field_name}' must have [label] or [label alias]"),
        start_offset: start,
        end_offset: start + len,
      }),
      db,
    );
  }

  if has_qualified_alias {
    let (start, len) = (field.syntax().offset(), field.syntax().text_len());
    salsa::Accumulator::accumulate(
      Diagnostics(Diagnostic::InvalidDeclarationFieldLabel {
        field_name: field_name.to_string(),
        message: format!(
          "qualified alias is not allowed on field '{field_name}', use 'label alias'"
        ),
        start_offset: start,
        end_offset: start + len,
      }),
      db,
    );
  }
}

/// Validate that positional [arg: N] indices increment contiguously starting from 0
pub fn validate_contiguous_arg_index(
  db: &dyn Database,
  field: &ElementFieldDeclaration,
  field_name: &str,
  arg_index: Option<usize>,
  expected_arg_index: &mut usize,
) {
  if let Some(found_idx) = arg_index {
    if found_idx != *expected_arg_index {
      let (start, len) = (field.syntax().offset(), field.syntax().text_len());
      salsa::Accumulator::accumulate(
        Diagnostics(Diagnostic::NonContiguousFieldArgIndex {
          field_name: field_name.to_string(),
          found_index: found_idx,
          expected_index: *expected_arg_index,
          start_offset: start,
          end_offset: start + len,
        }),
        db,
      );
    }
    *expected_arg_index = found_idx + 1;
  }
}

/// Validate that a constraint entry has a non-empty name and a closure RHS
pub fn validate_constraint_entry(
  db: &dyn Database,
  attr: &crate::ast::ElementAttributeDeclaration,
) -> Option<crate::db::types::ConstraintDefinition> {
  let name_node = attr.name().and_then(|n| n.name())?;
  let constraint_name = name_node.as_name()?;
  let syntax = attr.syntax();
  let (start, len) = (syntax.offset(), syntax.text_len());

  if constraint_name.trim().is_empty() {
    salsa::Accumulator::accumulate(
      Diagnostics(Diagnostic::InvalidConstraintDefinition {
        constraint_name: constraint_name.clone(),
        message: "constraint name cannot be empty".to_string(),
        start_offset: start,
        end_offset: start + len,
      }),
      db,
    );
    return None;
  }

  let closure_expr = attr
    .value()
    .and_then(|v| v.expr())
    .map(|e| e.unwrap_parens())
    .and_then(|e| crate::ast::ClosureExpr::cast(e.syntax().clone()));

  if let Some(closure) = closure_expr {
    Some(crate::db::types::ConstraintDefinition {
      name: constraint_name,
      closure_syntax: closure.syntax().clone(),
    })
  } else {
    salsa::Accumulator::accumulate(
      Diagnostics(Diagnostic::InvalidConstraintDefinition {
        constraint_name: constraint_name.clone(),
        message: format!("constraint '{constraint_name}' value must be a closure expression"),
        start_offset: start,
        end_offset: start + len,
      }),
      db,
    );
    None
  }
}
