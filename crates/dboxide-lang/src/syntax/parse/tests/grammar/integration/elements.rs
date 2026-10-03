use crate::syntax::parse::tests::utils::*;

/* element_declaration */

#[test]
fn comprehensive_element_declaration() {
  let input = r#"Table {

}

TableGroup group {

}

Ref {

}

Note: 'This is a note'

Note: '''This is 
another note'''

Table Users as U {
    
}"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "TableGroup")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "group"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "Ref")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "\n"
      "}"))
  "\n"
  "\n"
  (InlineElementDeclaration
    (ElementDeclarationType
      "Note")
    ":"
    " "
    (ElementFieldDeclaration
      (ElementFieldDeclarationArg
        (SqStringExpr
          "'This is a note'"))))
  "\n"
  "\n"
  (InlineElementDeclaration
    (ElementDeclarationType
      "Note")
    ":"
    " "
    (ElementFieldDeclaration
      (ElementFieldDeclarationArg
        (TqStringExpr
          "'''This is \nanother note'''"))))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Users"))
    " "
    "as"
    " "
    (ElementDeclarationAlias
      (IdentExpr
        "U"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

/* element_in_simple_body */

#[test]
fn comprehensive_element_in_simple_body() {
  let input = r#"Note: Enum E {}"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (InlineElementDeclaration
    (ElementDeclarationType
      "Note")
    ":"
    " "
    (BlockElementDeclaration
      (ElementDeclarationType
        "Enum")
      " "
      (ElementDeclarationTargetFragment
        (IdentExpr
          "E"))
      " "
      (BlockElementDeclarationBody
        "{"
        "}")))
  "")"#;
  assert_eq!(tree, expected);
}

/* nested_element */

#[test]
fn comprehensive_nested_element() {
  let input = r#"Project {
    Table A {

    }

    Table B as C {
        
    }
}

Table wrong_nested_element [] {
    Indexes wrong nested element {} // parsed successfully as function application
}
"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationType
      "Project")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (BlockElementDeclaration
        (ElementDeclarationType
          "Table")
        " "
        (ElementDeclarationTargetFragment
          (IdentExpr
            "A"))
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "\n"
          "    "
          "}"))
      "\n"
      "\n"
      "    "
      (BlockElementDeclaration
        (ElementDeclarationType
          "Table")
        " "
        (ElementDeclarationTargetFragment
          (IdentExpr
            "B"))
        " "
        "as"
        " "
        (ElementDeclarationAlias
          (IdentExpr
            "C"))
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "        "
          "\n"
          "    "
          "}"))
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationType
      "Table")
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "wrong_nested_element"))
    " "
    (SettingList
      "["
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (BlockElementDeclaration
        (ElementDeclarationType
          "Indexes")
        " "
        (ElementDeclarationTargetFragment
          (IdentExpr
            "wrong"))
        " "
        (ElementDeclarationTargetFragment
          (IdentExpr
            "nested"))
        " "
        (BlockElementDeclarationBody
          (Error
            "element")))
      " "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (Error
            (Error
              "{"))))
      "}"))
  " "
  "// parsed successfully as function application"
  "\n"
  (Error
    "}")
  "\n"
  "")"#;
  assert_eq!(tree, expected);
}
