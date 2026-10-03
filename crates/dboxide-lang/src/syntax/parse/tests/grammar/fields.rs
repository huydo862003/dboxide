use crate::syntax::parse::tests::utils::*;

#[test]
fn field_csv_basic() {
  let tree = parse_source(
    r#"Records r {
  1, 'Alice', 2
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            (SqStringExpr
              " "
              "'Alice'")
            ","
            (NumberExpr
              " "
              "2"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_with_settings() {
  let tree = parse_source(
    r#"Records r {
  1, 'Alice' [note: 'test']
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            (SqStringExpr
              " "
              "'Alice'")))
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
                "'test'")))
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_stops_at_newline() {
  let tree = parse_source(
    r#"Records r {
  1, 2
  3, 4
}"#,
  );
  // Each line is a separate field declaration; csv does not span lines
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            (NumberExpr
              " "
              "2"))))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "3")
            ","
            (NumberExpr
              " "
              "4"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_trailing_comma_is_ignored() {
  let tree = parse_source(
    r#"Records r {
  1, 2,
}"#,
  );
  // Trailing comma before `}` stops the csv without an extra element
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            (NumberExpr
              " "
              "2")
            ",")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_double_quoted_string() {
  let tree = parse_source(
    r#"Records r {
  1, "Alice"
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            (DqStringExpr
              " "
              "\"Alice\""))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_empty_fields() {
  let tree = parse_source(
    r#"Records r {
  1,,2
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            ","
            (NumberExpr
              "2"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_leading_comma() {
  let tree = parse_source(
    r#"Records r {
  ,1,2
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            ","
            (NumberExpr
              "1")
            ","
            (NumberExpr
              "2"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_only_commas() {
  // Single comma means 2 null slots
  let tree = parse_source(
    r#"Records r {
  ,
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            ",")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_multiple_empty_fields() {
  let tree = parse_source(
    r#"Records r {
  1,,,2,,
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            ","
            ","
            (NumberExpr
              "2")
            ","
            ",")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_null_and_booleans() {
  let tree = parse_source(
    r#"Records r {
  null, true, false
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (IdentExpr
              "null")
            ","
            (IdentExpr
              " "
              "true")
            ","
            (IdentExpr
              " "
              "false"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_negative_number() {
  let tree = parse_source(
    r#"Records r {
  -1, -2
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (PrefixExpr
              "-"
              (NumberExpr
                "1"))
            ","
            (PrefixExpr
              " "
              "-"
              (NumberExpr
                "2")))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_oq_string() {
  let tree = parse_source(
    r#"Records r {
  1, `now()`
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            (OqStringExpr
              " "
              "`now()`"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_single_value_is_plain_arg() {
  // A single value with no comma is not a CsvExpr - it's a plain arg
  let tree = parse_source(
    r#"Records r {
  42
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (NumberExpr
            "42")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_stops_at_bracket() {
  // `[` starts a setting list, so CSV stops before it
  let tree = parse_source(
    r#"Records r {
  1, 2 [note: 'x']
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Records"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "r"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            (NumberExpr
              " "
              "2")))
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
          "]"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_csv_as_second_arg() {
  // CSV after a discriminator arg: first arg is plain, rest become one CsvExpr
  let tree = parse_source(
    r#"Table t {
  col 1, 2
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
            "col"))
        " "
        (ElementFieldDeclarationArg
          (CsvExpr
            (NumberExpr
              "1")
            ","
            (NumberExpr
              " "
              "2"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn field_basic() {
  let tree = parse_source(
    r#"Table t {
  id integer
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
fn multiple_fields() {
  let tree = parse_source(
    "Table t {
  id int
  name varchar
}",
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
      (BlockElementDeclaration
        (ElementDeclarationTyp
          (IdentExpr
            "indexes"))
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
