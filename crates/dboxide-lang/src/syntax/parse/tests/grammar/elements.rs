use crate::syntax::parse::tests::utils::*;

#[test]
fn block_element_basic() {
  let tree = parse_source("Table users {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn block_element_with_alias() {
  let tree = parse_source("Table users as u {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    "as"
    " "
    (BlockElementDeclarationAlias
      "u")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn block_element_with_settings() {
  let tree = parse_source("Table users [note: 'main'] {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    (SettingList
      "["
      (SettingListItem
        (SettingListItemName
          "note")
        ":"
        (SettingListItemValue
          " "
          "'main'"))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn block_element_qualified_name() {
  let tree = parse_source("public.auth.User {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "public"
      "."
      "auth"
      "."
      "User")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn inline_element() {
  let tree = parse_source("Ref: orders.user_id > users.id");
  let expected = r#"(SourceFile
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Ref")
    ":"
    " "
    (ElementFieldDeclaration
      (InfixExpr
        (InfixExpr
          (IdentExpr
            "orders")
          "."
          (IdentExpr
            "user_id"))
        " "
        ">"
        (InfixExpr
          (IdentExpr
            " "
            "users")
          "."
          (IdentExpr
            "id")))))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn nested_element() {
  let tree = parse_source(
    "Table users {
  indexes {
  }
}",
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (BlockElementDeclaration
        (BlockElementDeclarationType
          "indexes")
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "  "
          "}"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn empty_block_body() {
  let tree = parse_source("Table t {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn multiple_top_level_elements() {
  let tree = parse_source(
    "Table a {}
Table b {}",
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "a"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "b"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn empty_input() {
  let tree = parse_source("");
  let expected = r#"(SourceFile
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn two_target_fragments() {
  let tree = parse_source("Table public users {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "public"))
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn two_qualified_target_fragments() {
  let tree = parse_source(r#"Table "public"."schema" users {}"#);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (InfixExpr
        (DqStringExpr
          "\"public\"")
        "."
        (DqStringExpr
          "\"schema\"")))
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn deeply_nested() {
  let tree = parse_source(
    r#"Table t {
  indexes {
    checks {
    }
  }
}"#,
  );
  assert!(tree.contains("BlockElementDeclarationBody"));
  // 3 levels of nesting = 3 BlockElementDeclaration
  let count = tree.matches("BlockElementDeclaration\n").count();
  assert_eq!(count, 3);
}

#[test]
fn round_trip_full_schema() {
  let input = r#"Table users as u [note: 'main'] {
  id integer [pk, increment]
  name varchar(255) [not null]
  email varchar [unique]
  created_at timestamp [default: `now()`]
  Note: 'User accounts'

  indexes {
    email [unique]
    (name, email)
  }
}

Enum status {
  active
  inactive
  pending
}

Ref: orders.user_id > users.id"#;
  let tree = parse_source(input);
  assert!(tree.contains("BlockElementDeclaration"));
  assert!(tree.contains("InlineElementDeclaration"));
  assert!(tree.contains("SettingList"));
}

#[test]
fn block_element_double_quoted_name_and_alias() {
  let (tree, diags) = parse_source_with_diagnostics(r#"Table "my users" as "U" {}"#);
  assert!(diags.is_empty());
  assert!(tree.contains(
    r#"(BlockElementDeclarationTargetFragment
      (DqStringExpr
        "\"my users\""))"#
  ));
  assert!(tree.contains(
    r#"(BlockElementDeclarationAlias
      "\"U\"")"#
  ));
}

#[test]
fn block_element_rejects_single_quoted_name_and_alias() {
  let (_tree, diags) = parse_source_with_diagnostics("Table 'users' as 'u' {}");
  assert!(!diags.is_empty());
}
