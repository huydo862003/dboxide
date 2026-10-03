use crate::syntax::parse::tests::utils::*;

#[test]
fn block_element_basic() {
  let tree = parse_source("Table users {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
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
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    "as"
    " "
    (ElementDeclarationAlias
      (IdentExpr
        "u"))
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
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
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
          (SqStringExpr
            " "
            "'main'")))
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
    (ElementDeclarationTyp
      (InfixExpr
        (InfixExpr
          (IdentExpr
            "public")
          "."
          (IdentExpr
            "auth"))
        "."
        (IdentExpr
          "User")))
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
    (ElementDeclarationTyp
      (IdentExpr
        "Ref"))
    ":"
    " "
    (ElementFieldDeclaration
      (ElementFieldDeclarationArg
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
              "id"))))))
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
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          (IdentExpr
            "indexes"))
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

#[test]
fn multiple_top_level_elements() {
  let tree = parse_source(
    "Table a {}
Table b {}",
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "a"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
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
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "public"))
    " "
    (ElementDeclarationTargetFragment
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
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (InfixExpr
        (DqStringExpr
          "\"public\"")
        "."
        (DqStringExpr
          "\"schema\"")))
    " "
    (ElementDeclarationTargetFragment
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
  let expected = r#"(SourceFile
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
      "\n"
      "  "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          (IdentExpr
            "indexes"))
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "    "
          (BlockElementDeclaration
            (ElementDeclarationTyp
              (IdentExpr
                "checks"))
            " "
            (BlockElementDeclarationBody
              "{"
              "\n"
              "    "
              "}"))
          "\n"
          "  "
          "}"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
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
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    "as"
    " "
    (ElementDeclarationAlias
      (IdentExpr
        "u"))
    " "
    (SettingList
      "["
      (SettingListItem
        (SettingListItemName
          "note")
        ":"
        (SettingListItemValue
          (SqStringExpr
            " "
            "'main'")))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "integer"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "pk"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "increment"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "name"))
        " "
        (ElementFieldDeclarationArg
          (CallExpr
            (IdentExpr
              "varchar")
            "("
            (NumberExpr
              "255")
            ")"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "not"
              " "
              "null"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "email"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "varchar"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "unique"))
          "]"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "created_at"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "timestamp"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              (OqStringExpr
                " "
                "`now()`")))
          "]"))
      "\n"
      "  "
      (ElementAttributeDeclaration
        (ElementAttributeDeclarationName
          (IdentExpr
            "Note"))
        ":"
        (ElementAttributeDeclarationValue
          (SqStringExpr
            " "
            "'User accounts'")))
      "\n"
      "\n"
      "  "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          (IdentExpr
            "indexes"))
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "    "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (IdentExpr
                "email"))
            " "
            (SettingList
              "["
              (SettingListItem
                (SettingListItemName
                  "unique"))
              "]"))
          "\n"
          "    "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (TupleExpr
                "("
                (IdentExpr
                  "name")
                ","
                (IdentExpr
                  " "
                  "email")
                ")")))
          "\n"
          "  "
          "}"))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Enum"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "status"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "active")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "inactive")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "pending")))
      "\n"
      "}"))
  "\n"
  "\n"
  (InlineElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Ref"))
    ":"
    " "
    (ElementFieldDeclaration
      (ElementFieldDeclarationArg
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
              "id"))))))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn block_element_double_quoted_name_and_alias() {
  let (tree, diagnostics) = parse_source_with_diagnostics(r#"Table "my users" as "U" {}"#);
  assert!(diagnostics.is_empty());
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (DqStringExpr
        "\"my users\""))
    " "
    "as"
    " "
    (ElementDeclarationAlias
      (DqStringExpr
        "\"U\""))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn block_element_rejects_single_quoted_name_and_alias() {
  let (_tree, diagnostics) = parse_source_with_diagnostics("Table 'users' as 'u' {}");
  assert!(!diagnostics.is_empty());
}
