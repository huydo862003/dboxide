use crate::syntax::parse::tests::utils::*;

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
