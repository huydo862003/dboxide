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
    (ElementDeclarationTyp
      "Test")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "CallExpression"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
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
                "3"))))
        " "
        (ElementFieldDeclarationArg
          (TupleExpr
            "("
            ")"))
        " "
        (ElementFieldDeclarationArg
          (TupleExpr
            "("
            ")")))
      "\n"
      "    "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (PrefixExpr
            "-"
            (CallExpr
              (NumberExpr
                "2")
              "("
              ")"))))
      "\n"
      "    "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (InfixExpr
            (IdentExpr
              "a")
            "."
            (CallExpr
              (IdentExpr
                "b")
              "("
              ")"))))
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
    (ElementDeclarationTyp
      "Test")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "FunctionApplication"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (IdentExpr
            "id"))
        "\t"
        (ElementFieldDeclarationArg
          (IdentExpr
            "integer"))
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
        (ElementFieldDeclarationArg
          (IdentExpr
            "name"))
        " "
        (ElementFieldDeclarationArg
          (IdentExpr
            "char"))
        " "
        (ElementFieldDeclarationArg
          (ParenExpr
            "("
            (NumberExpr
              "255")
            ")"))
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
