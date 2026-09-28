use crate::syntax::parse::tests::utils::*;
use crate::types::diagnostics::Diagnostic;

#[test]
fn unclosed_brace() {
  let (_, diags) = parse_source_with_diagnostics("Table t {");
  assert!(
    diags
      .iter()
      .any(|d| matches!(d, Diagnostic::UnclosedDelimiter { delimiter: "{", .. }))
  );
}

#[test]
fn unclosed_bracket() {
  let (_, diags) = parse_source_with_diagnostics(
    "Table t {
  id int [pk
}",
  );
  assert!(
    diags
      .iter()
      .any(|d| matches!(d, Diagnostic::UnclosedDelimiter { delimiter: "[", .. }))
  );
}

#[test]
fn unclosed_tuple() {
  let (_, diags) = parse_source_with_diagnostics(
    "Table t {
  (col1, col2
}",
  );
  assert!(
    diags
      .iter()
      .any(|d| matches!(d, Diagnostic::UnclosedDelimiter { delimiter: "(", .. }))
  );
}

#[test]
fn unexpected_token_at_top_level() {
  let (_, diags) = parse_source_with_diagnostics("123");
  assert!(!diags.is_empty());
}

#[test]
fn multiple_errors() {
  let (_, diags) = parse_source_with_diagnostics(
    "123
456
789",
  );
  assert!(diags.len() >= 3);
}

#[test]
fn recovery_continues_after_error() {
  let tree = parse_source(
    "123
Table t {}",
  );
  assert!(tree.contains("BlockElementDeclaration"));
  assert!(tree.contains("Error"));
}
