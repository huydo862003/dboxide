use crate::syntax::parse::tests::utils::*;

#[test]
fn field_basic() {
  let tree = parse_source(
    r#"Table t {
  id integer
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
            "integer")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_with_settings() {
  let tree = parse_source(
    r#"Table t {
  id int [pk]
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
fn multiple_fields() {
  let tree = parse_source(
    "Table t {
  id int
  name varchar
}",
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
            "int")))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "name"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "varchar")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_with_tuple() {
  let tree = parse_source(
    r#"Table t {
  indexes {
    (col1, col2) [unique]
  }
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
        (ElementDeclarationType
          "indexes")
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "    "
          (ElementFieldDeclaration
            (ElementFieldDeclarationArg
              (TupleExpr
                "("
                (IdentExpr
                  "col1")
                ","
                (IdentExpr
                  " "
                  "col2")
                ")"))
            " "
            (SettingList
              "["
              (SettingListItem
                (SettingListItemName
                  "unique"))
              "]"))
          "\n"
          "  "
          "}"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_call_without_space() {
  let tree = parse_source(
    r#"Table t {
  name varchar(255)
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
          (CallExpr
            (IdentExpr
              "varchar")
            "("
            (NumberExpr
              "255")
            ")")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_call_with_space_is_not_call() {
  let tree = parse_source(
    r#"Table t {
  name varchar (255)
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
        (ElementFieldDeclarationArg
          (ParenExpr
            "("
            (NumberExpr
              "255")
            ")")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_index_without_space() {
  let tree = parse_source(
    r#"Table t {
  arr int[10]
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
            "arr"))
        " "
        (ElementFieldDeclarationArg
          (IndexExpr
            (IdentExpr
              "int")
            "["
            (NumberExpr
              "10")
            "]")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_index_with_space_is_setting() {
  let tree = parse_source(
    r#"Table t {
  arr int [10]
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
            "arr"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "int"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              (Error
                "10")))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_with_binary_operator() {
  let tree = parse_source(
    r#"Table t {
  id int = 1
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
          (InfixExpr
            (IdentExpr
              "int")
            " "
            "="
            (NumberExpr
              " "
              "1"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_with_complex_expression() {
  let tree = parse_source(
    r#"Table t {
  1 + 2 * 3
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
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
          (InfixExpr
            (NumberExpr
              "1")
            " "
            "+"
            (InfixExpr
              (NumberExpr
                " "
                "2")
              " "
              "*"
              (NumberExpr
                " "
                "3")))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}
