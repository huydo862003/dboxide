use crate::syntax::parse::tests::utils::*;
use crate::types::diagnostics::DiagnosticKind;

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
    r#"use {
  table my_table
  enum my_enum as MY_ENUM
} from './db.dbml'"#,
  );
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (UseSpecifierList
      "{"
      "\n"
      "  "
      (UseSpecifier
        (UseDeclarationTargetFragment
          (IdentExpr
            "table"))
        " "
        (UseDeclarationTargetFragment
          (IdentExpr
            "my_table")))
      "\n"
      "  "
      (UseSpecifier
        (UseDeclarationTargetFragment
          (IdentExpr
            "enum"))
        " "
        (UseDeclarationTargetFragment
          (IdentExpr
            "my_enum"))
        " "
        "as"
        (IdentExpr
          (UseSpecifierAlias
            (IdentExpr
              " "
              "MY_ENUM"))))
      "\n"
      "}")
    " "
    "from"
    " "
    "'./db.dbml'")
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn reuse_wildcard() {
  let tree = parse_source(r#"reuse * from './index.dbml'"#);
  let expected = r#"(SourceFile
  (UseDeclaration
    "reuse"
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
fn use_specifiers_double_quoted_identifier() {
  let (tree, diagnostics) = parse_source_with_diagnostics(
    r#"use {
  table "my table" as "MY TABLE"
} from './db.dbml'"#,
  );
  assert!(diagnostics.is_empty());
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (UseSpecifierList
      "{"
      "\n"
      "  "
      (UseSpecifier
        (UseDeclarationTargetFragment
          (IdentExpr
            "table"))
        " "
        (UseDeclarationTargetFragment
          (DqStringExpr
            "\"my table\""))
        " "
        "as"
        (DqStringExpr
          (UseSpecifierAlias
            (DqStringExpr
              " "
              "\"MY TABLE\""))))
      "\n"
      "}")
    " "
    "from"
    " "
    "'./db.dbml'")
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn use_specifiers_rejects_single_quoted_identifier() {
  let (_tree, diagnostics) = parse_source_with_diagnostics(
    r#"use {
  table 'my_table' as 'my_alias'
} from './db.dbml'"#,
  );
  assert!(!diagnostics.is_empty());
}

#[test]
fn use_specifiers_synchronization_recovery() {
  let (tree, diagnostics) = parse_source_with_diagnostics(
    r#"use {
  ??? invalid specifier
  table valid_table
} from './db.dbml'"#,
  );
  assert!(!diagnostics.is_empty());
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (UseSpecifierList
      "{"
      "\n"
      "  "
      (Error
        "???"
        " "
        "invalid"
        " "
        "specifier")
      "\n"
      "  "
      (UseSpecifier
        (UseDeclarationTargetFragment
          (IdentExpr
            "table"))
        " "
        (UseDeclarationTargetFragment
          (IdentExpr
            "valid_table")))
      "\n"
      "}")
    " "
    "from"
    " "
    "'./db.dbml'")
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn use_specifier_list_unclosed_at_eof() {
  let (tree, diagnostics) = parse_source_with_diagnostics(r#"use { table users"#);
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.kind,
    DiagnosticKind::UnclosedDelimiter { delimiter } if *delimiter == "{"
  )));
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (UseSpecifierList
      "{"
      " "
      (UseSpecifier
        (UseDeclarationTargetFragment
          (IdentExpr
            "table"))
        " "
        (UseDeclarationTargetFragment
          (IdentExpr
            "users")))))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn use_specifier_list_stops_at_from_keyword() {
  let tree = parse_source(
    r#"use {
  table users
} from './db.dbml'"#,
  );
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (UseSpecifierList
      "{"
      "\n"
      "  "
      (UseSpecifier
        (UseDeclarationTargetFragment
          (IdentExpr
            "table"))
        " "
        (UseDeclarationTargetFragment
          (IdentExpr
            "users")))
      "\n"
      "}")
    " "
    "from"
    " "
    "'./db.dbml'")
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn use_specifier_list_from_as_early_terminator() {
  let (tree, _diagnostics) = parse_source_with_diagnostics(r#"use { table users from './db.dbml'"#);
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (UseSpecifierList
      "{"
      " "
      (UseSpecifier
        (UseDeclarationTargetFragment
          (IdentExpr
            "table"))
        " "
        (UseDeclarationTargetFragment
          (IdentExpr
            "users")))
      " ")
    "from"
    " "
    "'./db.dbml'")
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn use_sync_stops_at_newline_and_continues() {
  let (tree, _diagnostics) = parse_source_with_diagnostics(
    r#"use {
  ??? bad
  table good_table
} from './db.dbml'"#,
  );
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (UseSpecifierList
      "{"
      "\n"
      "  "
      (Error
        "???"
        " "
        "bad")
      "\n"
      "  "
      (UseSpecifier
        (UseDeclarationTargetFragment
          (IdentExpr
            "table"))
        " "
        (UseDeclarationTargetFragment
          (IdentExpr
            "good_table")))
      "\n"
      "}")
    " "
    "from"
    " "
    "'./db.dbml'")
  "")"#;
  assert_eq!(tree, expected);
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
  let (_tree, diagnostics) = parse_source_with_diagnostics("use * notfrom './db'");
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.kind,
    DiagnosticKind::MissingExpectedToken { expected } if *expected == "'from'"
  )));
}

#[test]
fn use_specifier_bare_name_emits_missing_import_kind() {
  let (_tree, diagnostics) = parse_source_with_diagnostics("use { Foo } from './db'");
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.kind,
    DiagnosticKind::MissingImportKind { name } if name == "Foo"
  )));
}

#[test]
fn use_specifier_bare_name_multiple_emits_diagnostic_for_each() {
  let (_tree, diagnostics) = parse_source_with_diagnostics(
    r#"use {
  Foo
  Bar
} from './db'"#,
  );
  let missing: Vec<_> = diagnostics
    .iter()
    .filter(|diagnostic| matches!(&diagnostic.kind, DiagnosticKind::MissingImportKind { .. }))
    .collect();
  assert_eq!(
    missing.len(),
    2,
    "expected MissingImportKind for each bare name"
  );
}

#[test]
fn use_specifier_with_kind_no_missing_import_kind_diagnostic() {
  let (_tree, diagnostics) = parse_source_with_diagnostics("use { type Foo } from './db'");
  assert!(
    !diagnostics
      .iter()
      .any(|diagnostic| matches!(&diagnostic.kind, DiagnosticKind::MissingImportKind { .. })),
    "unexpected MissingImportKind for valid specifier"
  );
}
