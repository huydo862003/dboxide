use crate::syntax::parse::tests::utils::*;

/* Expression */

#[test]
fn comprehensive_expression() {
  let input = r#"Test Expression {
    **b
    
    1 + 2 * 3
    1 + 2 + 3
    1 + -2 + 3
    (1 - 2) + 3
    1 + 2.0 - 3.2

    1 +
    2 +
    3

    1
    + 2
    + 3

    a.b.c

    a.
    b.
    c

    a.b
     .c

    f()

    (1, 2, 3)
    (2, 3, 4)

    (f
     (1, 2, 3))

    (1,
     2)

    1 * 2 / 3 != 1 * (2 / 3)

    1 == 1

    a = 1 <= 2 + 3

    b = 1 == 1

    a != b + c ()

    +++----++-1
    ---++---+1
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
        "Expression"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (ElementFieldDeclaration
        (Error
          (Error
            "**"))
        (IdentExpr
          "b"))
      "\n"
      "    "
      "\n"
      "    "
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
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (NumberExpr
              "1")
            " "
            "+"
            (NumberExpr
              " "
              "2"))
          " "
          "+"
          (NumberExpr
            " "
            "3")))
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (NumberExpr
              "1")
            " "
            "+"
            (PrefixExpr
              " "
              "-"
              (NumberExpr
                "2")))
          " "
          "+"
          (NumberExpr
            " "
            "3")))
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (ParenExpr
            "("
            (InfixExpr
              (NumberExpr
                "1")
              " "
              "-"
              (NumberExpr
                " "
                "2"))
            ")")
          " "
          "+"
          (NumberExpr
            " "
            "3")))
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (NumberExpr
              "1")
            " "
            "+"
            (NumberExpr
              " "
              "2.0"))
          " "
          "-"
          (NumberExpr
            " "
            "3.2")))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (NumberExpr
            "1")
          " "
          "+"
          (Error)))
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (NumberExpr
            "2")
          " "
          "+"
          (Error)))
      "\n"
      "    "
      (ElementFieldDeclaration
        (NumberExpr
          "3"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (NumberExpr
          "1"))
      "\n"
      "    "
      (ElementFieldDeclaration
        (PrefixExpr
          "+"
          (NumberExpr
            " "
            "2")))
      "\n"
      "    "
      (ElementFieldDeclaration
        (PrefixExpr
          "+"
          (NumberExpr
            " "
            "3")))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (IdentExpr
              "a")
            "."
            (IdentExpr
              "b"))
          "."
          (IdentExpr
            "c")))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (IdentExpr
            "a")
          "."
          (Error)))
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (IdentExpr
            "b")
          "."
          (Error)))
      "\n"
      "    "
      (ElementFieldDeclaration
        (IdentExpr
          "c"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (IdentExpr
            "a")
          "."
          (IdentExpr
            "b")))
      "\n"
      "     "
      (ElementFieldDeclaration
        (Error
          (Error
            "."))
        (IdentExpr
          "c"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (CallExpr
          (IdentExpr
            "f")
          "("
          ")"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (TupleExpr
          "("
          (NumberExpr
            "1")
          ","
          (NumberExpr
            " "
            "2")
          ","
          (NumberExpr
            " "
            "3")
          ")"))
      "\n"
      "    "
      (ElementFieldDeclaration
        (TupleExpr
          "("
          (NumberExpr
            "2")
          ","
          (NumberExpr
            " "
            "3")
          ","
          (NumberExpr
            " "
            "4")
          ")"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (ParenExpr
          "("
          (CallExpr
            (IdentExpr
              "f")
            "\n"
            "     "
            "("
            (NumberExpr
              "1")
            ","
            (NumberExpr
              " "
              "2")
            ","
            (NumberExpr
              " "
              "3")
            ")")
          ")"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (TupleExpr
          "("
          (NumberExpr
            "1")
          ","
          (NumberExpr
            "\n"
            "     "
            "2")
          ")"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (InfixExpr
            (InfixExpr
              (NumberExpr
                "1")
              " "
              "*"
              (NumberExpr
                " "
                "2"))
            " "
            "/"
            (NumberExpr
              " "
              "3"))
          " "
          "!="
          (InfixExpr
            (NumberExpr
              " "
              "1")
            " "
            "*"
            (ParenExpr
              " "
              "("
              (InfixExpr
                (NumberExpr
                  "2")
                " "
                "/"
                (NumberExpr
                  " "
                  "3"))
              ")"))))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (NumberExpr
            "1")
          " "
          "=="
          (NumberExpr
            " "
            "1")))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (IdentExpr
            "a")
          " "
          "="
          (InfixExpr
            (NumberExpr
              " "
              "1")
            " "
            "<="
            (InfixExpr
              (NumberExpr
                " "
                "2")
              " "
              "+"
              (NumberExpr
                " "
                "3")))))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (IdentExpr
            "b")
          " "
          "="
          (InfixExpr
            (NumberExpr
              " "
              "1")
            " "
            "=="
            (NumberExpr
              " "
              "1"))))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (InfixExpr
          (IdentExpr
            "a")
          " "
          "!="
          (InfixExpr
            (IdentExpr
              " "
              "b")
            " "
            "+"
            (IdentExpr
              " "
              "c")))
        " "
        (TupleExpr
          "("
          ")"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (Error
          (Error
            "+++----++-"))
        (NumberExpr
          "1"))
      "\n"
      "    "
      (ElementFieldDeclaration
        (Error
          (Error
            "---++---+"))
        (NumberExpr
          "1"))
      "\n"
      "}"))
  "\n"
  "")"#;
  assert_eq!(tree, expected);
}

/* list_expression */

#[test]
fn comprehensive_list_expression() {
  let input = r#"Test ListExpression {
    id integer [one, two: 'two',
                three: 'three', primary 
                key, : 'invalid-attribute' ]
    abc " gibberish type " [ref: empty. ]
}"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Test")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "ListExpression"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (ElementFieldDeclaration
        (IdentExpr
          "id")
        " "
        (IdentExpr
          "integer")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "one"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "two")
            ":"
            (SettingListItemValue
              " "
              "'two'"))
          ","
          "\n"
          "                "
          (SettingListItem
            (SettingListItemName
              "three")
            ":"
            (SettingListItemValue
              " "
              "'three'"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              "primary"))
          " "
          "\n"
          "                "
          (SettingListItem
            (SettingListItemName
              "key"))
          ","
          " "
          (SettingListItem
            (SettingListItemName
              (Error
                ":"
                " "
                "'invalid-attribute'"
                " ")))
          "]"))
      "\n"
      "    "
      (ElementFieldDeclaration
        (IdentExpr
          "abc")
        " "
        (DqStringExpr
          "\" gibberish type \"")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "ref")
            ":"
            (SettingListItemValue
              " "
              "empty"
              "."
              " "))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

/* literal_element_expression */

#[test]
fn comprehensive_literal_element_expression() {
  let input = r#"Test LiteralElementExpression {
    indexes [note: 'this is an index element'] {
        (`id * 2`, id) [primary key]
        name [unique]
    }

    Note {
        '''
            this is a note
        '''
    }
}"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Test")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "LiteralElementExpression"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (BlockElementDeclaration
        (BlockElementDeclarationType
          "indexes")
        " "
        (SettingList
          "["
          (SettingListItem
            (SettingListItemName
              "note")
            ":"
            (SettingListItemValue
              " "
              "'this is an index element'"))
          "]")
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "        "
          (ElementFieldDeclaration
            (TupleExpr
              "("
              (OqStringExpr
                "`id * 2`")
              ","
              (IdentExpr
                " "
                "id")
              ")")
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
          "        "
          (ElementFieldDeclaration
            (IdentExpr
              "name")
            " "
            (SettingList
              "["
              (SettingListItem
                (SettingListItemName
                  "unique"))
              "]"))
          "\n"
          "    "
          "}"))
      "\n"
      "\n"
      "    "
      (BlockElementDeclaration
        (BlockElementDeclarationType
          "Note")
        " "
        (BlockElementDeclarationBody
          "{"
          "\n"
          "        "
          (ElementFieldDeclaration
            (TqStringExpr
              "'''\n            this is a note\n        '''"))
          "\n"
          "    "
          "}"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

/* tuple_expression */

#[test]
fn comprehensive_tuple_expression() {
  let input = r#"Test TupleExpression {
    ()
    (

    )

    (1, 
    2,
    3,)

    (1 - 2, 3 * 4, 5 / 6, 1 == 2, 1 >=
    3, 1 <=
    12)
}"#;
  let tree = parse_source(input);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (BlockElementDeclarationType
      "Test")
    " "
    (BlockElementDeclarationTargetFragment
      (IdentExpr
        "TupleExpression"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "    "
      (ElementFieldDeclaration
        (TupleExpr
          "("
          ")"))
      "\n"
      "    "
      (ElementFieldDeclaration
        (TupleExpr
          "("
          "\n"
          "\n"
          "    "
          ")"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (TupleExpr
          "("
          (NumberExpr
            "1")
          ","
          (NumberExpr
            " "
            "\n"
            "    "
            "2")
          ","
          (NumberExpr
            "\n"
            "    "
            "3")
          ","
          ")"))
      "\n"
      "\n"
      "    "
      (ElementFieldDeclaration
        (TupleExpr
          "("
          (InfixExpr
            (NumberExpr
              "1")
            " "
            "-"
            (NumberExpr
              " "
              "2"))
          ","
          (InfixExpr
            (NumberExpr
              " "
              "3")
            " "
            "*"
            (NumberExpr
              " "
              "4"))
          ","
          (InfixExpr
            (NumberExpr
              " "
              "5")
            " "
            "/"
            (NumberExpr
              " "
              "6"))
          ","
          (InfixExpr
            (NumberExpr
              " "
              "1")
            " "
            "=="
            (NumberExpr
              " "
              "2"))
          ","
          (InfixExpr
            (NumberExpr
              " "
              "1")
            " "
            ">="
            (NumberExpr
              "\n"
              "    "
              "3"))
          ","
          (InfixExpr
            (NumberExpr
              " "
              "1")
            " "
            "<="
            (NumberExpr
              "\n"
              "    "
              "12"))
          ")"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}
