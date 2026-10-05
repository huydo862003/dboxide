use crate::syntax::parse::tests::utils::*;
use crate::types::diagnostics::Diagnostic;

#[test]
fn use_wildcard() {
  let tree = parse_source("use * from './index.dbml'");
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (Wildcard
      "*")
    " "
    "from"
    " "
    "'./index.dbml'")
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn use_specifiers() {
  let tree = parse_source(
    "use {
  table my_table
  enum my_enum as MY_ENUM
} from './db.dbml'",
  );
  assert!(tree.contains("UseSpecifierList"));
  assert!(tree.contains("UseSpecifier"));
  assert!(tree.contains("my_table"));
  assert!(tree.contains("MY_ENUM"));
}

#[test]
fn reuse_wildcard() {
  let tree = parse_source("reuse * from './index.dbml'");
  assert!(tree.contains("UseDeclaration"));
  assert!(tree.contains("reuse"));
  assert!(tree.contains("Wildcard"));
}

#[test]
fn use_specifiers_double_quoted_identifier() {
  let (tree, diags) = parse_source_with_diagnostics(
    r#"use {
  table "my table" as "MY TABLE"
} from './db.dbml'"#,
  );
  assert!(diags.is_empty());
  assert!(tree.contains(r#""\"my table\"""#));
  assert!(tree.contains(r#""\"MY TABLE\"""#));
}

#[test]
fn use_specifiers_rejects_single_quoted_identifier() {
  let (_tree, diags) = parse_source_with_diagnostics(
    r#"use {
  table 'my_table' as 'my_alias'
} from './db.dbml'"#,
  );
  assert!(!diags.is_empty());
}

#[test]
fn use_specifiers_synchronization_recovery() {
  let (tree, diags) = parse_source_with_diagnostics(
    r#"use {
  ??? invalid specifier
  table valid_table
} from './db.dbml'"#,
  );
  assert!(!diags.is_empty());
  assert!(tree.contains("valid_table"));
  assert!(tree.contains("'./db.dbml'"));
}

#[test]
fn use_specifier_list_unclosed_at_eof() {
  // Missing closing `}` and `from ...` - should emit UnclosedDelimiter
  let (tree, diags) = parse_source_with_diagnostics("use { table users");
  assert!(diags.iter().any(|d| matches!(
    d,
    Diagnostic::UnclosedDelimiter { delimiter, .. } if *delimiter == "{"
  )));
  // Should still recover and include what was parsed
  assert!(tree.contains("UseSpecifier"));
  assert!(tree.contains("users"));
}

#[test]
fn use_specifier_list_stops_at_from_keyword() {
  // "from" inside specifier list terminates the list, not a specifier name
  let tree = parse_source(
    r#"use {
  table users
} from './db.dbml'"#,
  );
  assert!(tree.contains("UseDeclaration"));
  assert!(tree.contains("users"));
  assert!(tree.contains("'./db.dbml'"));
  // "from" should appear as a keyword token, not as a UseSpecifier name
  assert!(!tree.contains("UseSpecifierKind\n      \"from\""));
}

#[test]
fn use_specifier_list_from_as_early_terminator() {
  // "from" encountered while parsing specifier items (without closing brace first)
  // This tests the guard that breaks out of the loop on "from" keyword
  let (tree, _diags) = parse_source_with_diagnostics("use { table users from './db.dbml'");
  // Should parse as use declaration (from terminates the specifier list)
  assert!(tree.contains("UseDeclaration"));
}

#[test]
fn use_sync_stops_at_newline_and_continues() {
  let (tree, _diags) = parse_source_with_diagnostics(
    r#"use {
  ??? bad
  table good_table
} from './db.dbml'"#,
  );
  // Sync should stop at newline, allowing good_table to be parsed
  assert!(tree.contains("good_table"));
  assert!(tree.contains("'./db.dbml'"));
}

#[test]
fn reuse_is_accepted_keyword() {
  let tree = parse_source("reuse * from './index.dbml'");
  assert!(tree.contains("UseDeclaration"));
  assert!(tree.contains("reuse"));
}

#[test]
fn use_wildcard_full_structure() {
  let tree = parse_source("use * from './index.dbml'");
  assert_eq!(
    tree,
    r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (Wildcard
      "*")
    " "
    "from"
    " "
    "'./index.dbml'")
  "")"#
  );
}

#[test]
fn use_wrong_keyword_instead_of_from() {
  // Non-"from" ident after specifiers emits MissingExpectedToken for 'from'
  let (_tree, diags) = parse_source_with_diagnostics("use * notfrom './db'");
  assert!(diags.iter().any(|d| matches!(
    d,
    Diagnostic::MissingExpectedToken { expected, .. } if *expected == "'from'"
  )));
}
