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
    (ElementDeclarationTyp
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
    (ElementDeclarationTyp
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
    (ElementDeclarationTyp
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
    (ElementDeclarationTyp
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
    (ElementDeclarationTyp
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
    (ElementDeclarationTyp
      "Note")
    ":"
    " "
    (BlockElementDeclaration
      (ElementDeclarationTyp
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
    (ElementDeclarationTyp
      "Project")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (BlockElementDeclaration
        (ElementDeclarationTyp
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
        (ElementDeclarationTyp
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
    (ElementDeclarationTyp
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
        (ElementDeclarationTyp
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
      "indexes")
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
      "T")
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
          "indexes")
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
      "T")
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
          "Sub")
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
  let tree = parse_source("use { schema.table users } from './db.dbml'");
  assert!(tree.contains("UseSpecifier"));
  assert!(tree.contains("users"));
}

#[test]
fn inline_element_parsed_correctly() {
  // Inline (equality-form) element at top level
  let tree = parse_source("Ref: users.id > posts.user_id");
  assert!(tree.contains("ElementDeclarationTyp"));
}
