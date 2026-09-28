use crate::syntax::parse::tests::utils::*;

/* call_expression */

#[test]
fn comprehensive_call_expression() {
  let input = r#"Test CallExpression {
    1 ** 2 + 3 () ()
    -2()
    a.b()
}
"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Test")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "CallExpression"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (NumberExpr
            "1")
          " "
          "**"
          (InfixExpr
            (NumberExpr
              " "
              "2")
            " "
            "+"
            (NumberExpr
              " "
              "3")))
        " "
        (TupleExpr
          "("
          ")")
        " "
        (TupleExpr
          "("
          ")"))
      "\n"
      "    "
      (ElementFieldDeclaration
        (PrefixExpr
          "-"
          (CallExpr
            (NumberExpr
              "2")
            "("
            ")")))
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (IdentExpr
            "a")
          "."
          (CallExpr
            (IdentExpr
              "b")
            "("
            ")")))
      "\n"
      "}"))
  "\n"
  "")"#;
  assert_eq!(tree, expected);
}

/* function_application */

#[test]
fn comprehensive_function_application() {
  let input = r#"Test FunctionApplication {
    id	integer [primary key]
    name char (255) [unique]
}
"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Test")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "FunctionApplication"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        "\t"
        (IdentExpr
          "integer")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "primary"
              " "
              "key"))
          "]"))
      "\n"
      "    "
      (ElementFieldDeclaration
        (IdentExpr
          "name")
        " "
        (IdentExpr
          "char")
        " "
        (ParenExpr
          "("
          (NumberExpr
            "255")
          ")")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "unique"))
          "]"))
      "\n"
      "}"))
  "\n"
  "")"#;
  assert_eq!(tree, expected);
}
