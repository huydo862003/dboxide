use crate::syntax::parse::tests::utils::*;

#[test]
fn setting_simple() {
  let tree = parse_source(
    r#"Table t {
  id int [pk]
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
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "pk"))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn setting_with_value() {
  let tree = parse_source(
    r#"Table t {
  id int [default: 0]
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
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              (NumberExpr
                " "
                "0")))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn setting_multi_word_name() {
  let tree = parse_source(
    r#"Table t {
  id int [not null]
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
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
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
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn multiple_settings() {
  let tree = parse_source(
    r#"Table t {
  id int [pk, not null, increment]
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
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
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
              "not"
              " "
              "null"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "increment"))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn setting_backtick_value() {
  let tree = parse_source(
    r#"Table t {
  created_at timestamp [default: `now()`]
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
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn setting_double_quoted_name() {
  let (tree, diagnostics) = parse_source_with_diagnostics(
    r#"Table t {
  id int ["custom name": 'val']
}"#,
  );
  assert!(diagnostics.is_empty());
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
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "\"custom name\"")
            ":"
            (SettingListItemValue
              (SqStringExpr
                " "
                "'val'")))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn setting_rejects_single_quoted_name() {
  let (_tree, diagnostics) = parse_source_with_diagnostics(
    "Table t {
  id int ['custom name': 'val']
}",
  );
  assert!(!diagnostics.is_empty());
}

#[test]
fn setting_with_numeric_value() {
  let tree = parse_source(
    r#"Table t {
  id int [default: 42]
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
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              (NumberExpr
                " "
                "42")))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn setting_with_string_value() {
  let tree = parse_source(
    r#"Table t {
  name varchar [default: 'guest']
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
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "name"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "varchar"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "default")
            ":"
            (SettingListItemValue
              (SqStringExpr
                " "
                "'guest'")))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}
