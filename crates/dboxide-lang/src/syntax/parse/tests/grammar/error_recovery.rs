use crate::syntax::parse::tests::utils::*;
use crate::types::diagnostics::DiagnosticKind;

#[test]
fn unclosed_brace() {
  let (_, diagnostics) = parse_source_with_diagnostics("Table t {");
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    diagnostic.kind,
    DiagnosticKind::UnclosedDelimiter { delimiter: "{" }
  )));
}

#[test]
fn unclosed_bracket() {
  let (_, diagnostics) = parse_source_with_diagnostics(
    "Table t {
  id int [pk
}",
  );
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    diagnostic.kind,
    DiagnosticKind::UnclosedDelimiter { delimiter: "[" }
  )));
}

#[test]
fn unclosed_tuple() {
  let (_, diagnostics) = parse_source_with_diagnostics(
    "Table t {
  (col1, col2
}",
  );
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    diagnostic.kind,
    DiagnosticKind::UnclosedDelimiter { delimiter: "(" }
  )));
}

#[test]
fn unexpected_token_at_top_level() {
  let (tree, diagnostics) = parse_source_with_diagnostics("123");
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::UnexpectedToken {
        expected: "element declaration"
      }
      .into_error(0, 3)
    ]
  );
  assert_eq!(
    tree,
    r#"(SourceFile
  (Error
    "123")
  "")"#
  );
}

#[test]
fn multiple_errors() {
  let (tree, diagnostics) = parse_source_with_diagnostics(
    r#"123
456
789"#,
  );
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::UnexpectedToken {
        expected: "element declaration"
      }
      .into_error(0, 3),
      DiagnosticKind::UnexpectedToken {
        expected: "element declaration"
      }
      .into_error(4, 7),
      DiagnosticKind::UnexpectedToken {
        expected: "element declaration"
      }
      .into_error(8, 11),
    ]
  );
  assert_eq!(
    tree,
    r#"(SourceFile
  (Error
    "123")
  "\n"
  (Error
    "456")
  "\n"
  (Error
    "789")
  "")"#
  );
}

#[test]
fn recovery_continues_after_error() {
  let (tree, diagnostics) = parse_source_with_diagnostics(
    "123
Table t {}",
  );
  assert!(!diagnostics.is_empty());
  let expected = r#"(SourceFile
  (Error
    "123")
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}
