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
    (BlockElementDeclarationType
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
    (BlockElementDeclarationType
      "TableGroup")
    " "
    (BlockElementDeclarationTargetFragment
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
    (BlockElementDeclarationType
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
    (BlockElementDeclarationType
      "Note")
    ":"
    " "
    (ElementFieldDeclaration
      (SqStringExpr
        "'This is a note'")))
  "\n"
  "\n"
  (InlineElementDeclaration
    (BlockElementDeclarationType
      "Note")
    ":"
    " "
    (ElementFieldDeclaration
      (TqStringExpr
        "'''This is \nanother note'''")))
  "\n"
  "\n"
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "Users"))
    " "
    "as"
    " "
    (BlockElementDeclarationAlias
      "U")
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
    (BlockElementDeclarationType
      "Note")
    ":"
    " "
    (BlockElementDeclaration
      (BlockElementDeclarationType
        "Enum")
      " "
      (BlockElementDeclarationTargetFragment
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
    (BlockElementDeclarationType
      "Project")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (BlockElementDeclaration
        (BlockElementDeclarationType
          "Table")
        " "
        (BlockElementDeclarationTargetFragment
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
        (BlockElementDeclarationType
          "Table")
        " "
        (BlockElementDeclarationTargetFragment
          (IdentExpr
            "B"))
        " "
        "as"
        " "
        (BlockElementDeclarationAlias
          "C")
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
    (BlockElementDeclarationType
      "Table")
    " "
    (BlockElementDeclarationTargetFragment
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
        (BlockElementDeclarationType
          "Indexes")
        " "
        (BlockElementDeclarationTargetFragment
          (IdentExpr
            "wrong"))
        " "
        (BlockElementDeclarationTargetFragment
          (IdentExpr
            "nested"))
        " "
        (BlockElementDeclarationBody
          (Error
            "element")))
      " "
      (ElementFieldDeclaration
        (Error
          (Error
            "{")))
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
