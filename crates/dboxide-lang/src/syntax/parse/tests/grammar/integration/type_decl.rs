use crate::syntax::parse::tests::utils::*;

#[test]
fn type_block_empty() {
  let tree = parse_source("type Foo {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "Foo")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_block_with_role() {
  let tree = parse_source("type Foo [element] {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "Foo")
    " "
    (SettingList
      "["
      (SettingListItem
        (SettingListItemName
          "element"))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_alias_simple() {
  let tree = parse_source("type Str = string");
  let expected = r#"(SourceFile
  (EqualityDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "Str")
    " "
    "="
    " "
    (IdentExpr
      "string"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_alias_union() {
  let tree = parse_source(r#"type Status = "active" | "inactive""#);
  let expected = r#"(SourceFile
  (EqualityDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "Status")
    " "
    "="
    " "
    (InfixExpr
      (DqStringExpr
        "\"active\"")
      " "
      "|"
      (DqStringExpr
        " "
        "\"inactive\"")))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_with_field() {
  let tree = parse_source(
    r#"type Foo {
  name string
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "Foo")
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
            "string")))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_with_nested_fn() {
  let tree = parse_source(
    r#"type Foo {
  fn method(): bool {}
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "Foo")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (FuncDeclaration
        "fn"
        " "
        (FuncDeclarationName
          "method")
        (FuncDeclarationParams
          "("
          ")")
        (FuncDeclarationReturnTyp
          ":"
          " "
          (IdentExpr
            "bool"))
        " "
        (BlockElementDeclarationBody
          "{"
          "}"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_nested_inside_type() {
  let tree = parse_source(
    r#"type Outer {
  type Inner {}
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "Outer")
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          "type")
        " "
        (EqualityDeclarationName
          "Inner")
        " "
        (BlockElementDeclarationBody
          "{"
          "}"))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_role_multiple_settings() {
  let tree = parse_source("type T [element, on use: expand] {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "T")
    " "
    (SettingList
      "["
      (SettingListItem
        (SettingListItemName
          "element"))
      ","
      " "
      (SettingListItem
        (SettingListItemName
          "on"
          " "
          "use")
        ":"
        (SettingListItemValue
          (IdentExpr
            " "
            "expand")))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_quoted_name() {
  let tree = parse_source(r#"type "Foo" {}"#);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "\"Foo\"")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_error_missing_name() {
  let tree = parse_source("type {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName)
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_error_missing_body_or_eq() {
  let tree = parse_source("type Foo");
  let expected = r#"(SourceFile
  (EqualityDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "Foo"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_alias_call_expr() {
  let tree = parse_source("type T = foo()");
  let expected = r#"(SourceFile
  (EqualityDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "T")
    " "
    "="
    " "
    (CallExpr
      (IdentExpr
        "foo")
      "("
      ")"))
  "")"#;
  assert_eq!(tree, expected);
}
