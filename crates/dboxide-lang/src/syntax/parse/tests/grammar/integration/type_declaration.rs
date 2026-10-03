use crate::syntax::parse::tests::utils::*;

#[test]
fn typ_block_empty() {
  let tree = parse_source("type Foo {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Foo"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn typ_block_with_role() {
  let tree = parse_source("type Foo [element] {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Foo"))
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
fn typ_alias_simple() {
  let tree = parse_source("type Str = string");
  let expected = r#"(SourceFile
  (EqualityDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Str"))
    " "
    "="
    " "
    (IdentExpr
      "string"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn typ_alias_union() {
  let tree = parse_source(r#"type Status = "active" | "inactive""#);
  let expected = r#"(SourceFile
  (EqualityDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Status"))
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
fn typ_with_field() {
  let tree = parse_source(
    r#"type Foo {
  name string
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Foo"))
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
fn typ_with_nested_func() {
  let tree = parse_source(
    r#"type Foo {
  fn method(): bool {}
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Foo"))
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
fn typ_nested_inside_typ() {
  let tree = parse_source(
    r#"type Outer {
  type Inner {}
}"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Outer"))
    " "
    (BlockElementDeclarationBody
      "{"
      "\n"
      "  "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          (IdentExpr
            "type"))
        " "
        (ElementDeclarationTargetFragment
          (IdentExpr
            "Inner"))
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
fn typ_role_multiple_settings() {
  let tree = parse_source("type T [element, use: expand namespaces] {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "T"))
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
          "use")
        ":"
        (SettingListItemValue
          (IdentExpr
            " "
            "expand")
          (IdentExpr
            " "
            "namespaces")))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn typ_quoted_name() {
  let tree = parse_source(r#"type "Foo" {}"#);
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (DqStringExpr
        "\"Foo\""))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn typ_error_missing_name() {
  let tree = parse_source("type {}");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr))
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn typ_error_missing_body_or_eq() {
  let tree = parse_source("type Foo");
  let expected = r#"(SourceFile
  (EqualityDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Foo")))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn typ_alias_call_expr() {
  let tree = parse_source("type T = foo()");
  let expected = r#"(SourceFile
  (EqualityDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "T"))
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
