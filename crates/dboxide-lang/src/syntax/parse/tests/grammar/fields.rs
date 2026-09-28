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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "integer"))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int")
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "int"))
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "name")
        " "
        (IdentExpr
          "varchar"))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
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
          "    "
          (ElementFieldDeclaration
            (TupleExpr
              "("
              (IdentExpr
                "col1")
              ","
              (IdentExpr
                " "
                "col2")
              ")")
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "name")
        " "
        (CallExpr
          (IdentExpr
            "varchar")
          "("
          (NumberExpr
            "255")
          ")"))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "name")
        " "
        (IdentExpr
          "varchar")
        " "
        (ParenExpr
          "("
          (NumberExpr
            "255")
          ")"))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "arr")
        " "
        (IndexExpr
          (IdentExpr
            "int")
          "["
          (NumberExpr
            "10")
          "]"))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "arr")
        " "
        (IdentExpr
          "int")
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (InfixExpr
          (IdentExpr
            "int")
          " "
          "="
          (NumberExpr
            " "
            "1")))
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "t"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
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
              "3"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}
