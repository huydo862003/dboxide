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
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "\n"
      "}"))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "TableGroup"))
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
    (ElementDeclarationTyp
      (IdentExpr
        "Ref"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "\n"
      "}"))
  "\n"
  "\n"
  (InlineElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Note"))
    ":"
    " "
    (ElementFieldDeclaration
      (ElementFieldDeclarationArg
        (SqStringExpr
          "'This is a note'"))))
  "\n"
  "\n"
  (InlineElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Note"))
    ":"
    " "
    (ElementFieldDeclaration
      (ElementFieldDeclarationArg
        (TqStringExpr
          "'''This is \nanother note'''"))))
  "\n"
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
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
    (ElementDeclarationTyp
      (IdentExpr
        "Note"))
    ":"
    " "
    (BlockElementDeclaration
      (ElementDeclarationTyp
        (IdentExpr
          "Enum"))
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
    (ElementDeclarationTyp
      (IdentExpr
        "Project"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          (IdentExpr
            "Table"))
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
        (ElementDeclarationTyp
          (IdentExpr
            "Table"))
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
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
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
        (ElementDeclarationTyp
          (IdentExpr
            "Indexes"))
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

#[test]
fn block_element_with_settings_no_alias() {
  let tree = parse_source(
    r#"indexes [note: 'main'] {
  id
}"#,
  );
  assert_eq!(
    tree,
    r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "indexes"))
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
            "id")))
      "\n"
      "}"))
  "")"#
  );
}

#[test]
fn block_element_nested_with_settings_only() {
  // Nested block element identified by [settings] + { body }
  let tree = parse_source(
    r#"T E {
  indexes [note: 'x'] {}
}"#,
  );
  assert_eq!(
    tree,
    r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "T"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "E"))
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
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "note")
            ":"
            (SettingListItemValue
              (SqStringExpr
                " "
                "'x'")))
          "]")
        " "
        (BlockElementDeclarationBody
          "{"
          "}"))
      "\n"
      "}"))
  "")"#
  );
}

#[test]
fn block_element_head_with_nested_settings() {
  // Settings list contains nested brackets - scan_past_delimited depth tracking
  let tree = parse_source(
    r#"T E {
  Sub [settings: [a, b]] {}
}"#,
  );
  assert_eq!(
    tree,
    r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "T"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "E"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          (IdentExpr
            "Sub"))
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "settings")
            ":"
            (SettingListItemValue
              (ListExpr
                " "
                "["
                (IdentExpr
                  "a")
                ","
                (IdentExpr
                  " "
                  "b")
                "]")))
          "]")
        " "
        (BlockElementDeclarationBody
          "{"
          "}"))
      "\n"
      "}"))
  "")"#
  );
}

#[test]
fn qualified_name_in_use_specifier() {
  let tree = parse_source(r#"use { schema.table users } from './db.dbml'"#);
  let expected = r#"(SourceFile
  (UseDeclaration
    "use"
    " "
    (UseSpecifierList
      "{"
      " "
      (UseSpecifier
        (UseDeclarationTargetFragment
          (InfixExpr
            (IdentExpr
              "schema")
            "."
            (IdentExpr
              "table")))
        " "
        (UseDeclarationTargetFragment
          (IdentExpr
            "users")))
      " "
      "}")
    " "
    "from"
    " "
    "'./db.dbml'")
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn inline_element_parsed_correctly() {
  let tree = parse_source(r#"Ref: users.id > posts.user_id"#);
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
              "users")
            "."
            (IdentExpr
              "id"))
          " "
          ">"
          (InfixExpr
            (IdentExpr
              " "
              "posts")
            "."
            (IdentExpr
              "user_id"))))))
  "")"#;
  assert_eq!(tree, expected);
}
