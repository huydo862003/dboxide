use crate::syntax::parse::tests::utils::*;

#[test]
fn get_no_params_with_return() {
  let tree = parse_source("type T { namespace { get items(): Item[] {} } }");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "T")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          "namespace")
        " "
        (BlockElementDeclarationBody
          "{"
          " "
          (GetDeclaration
            "get"
            " "
            (GetDeclarationName
              "items")
            (FuncDeclarationParams
              "("
              ")")
            (FuncDeclarationReturnTyp
              ":"
              " "
              (IndexExpr
                (IdentExpr
                  "Item")
                "["
                "]"))
            " "
            (BlockElementDeclarationBody
              "{"
              "}"))
          " "
          "}"))
      " "
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn get_with_one_param() {
  let tree = parse_source("type T { namespace { get find(id: int): Item {} } }");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "T")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          "namespace")
        " "
        (BlockElementDeclarationBody
          "{"
          " "
          (GetDeclaration
            "get"
            " "
            (GetDeclarationName
              "find")
            (FuncDeclarationParams
              "("
              (FuncDeclarationParam
                (IdentExpr
                  "id")
                ":"
                " "
                (IdentExpr
                  "int"))
              ")")
            (FuncDeclarationReturnTyp
              ":"
              " "
              (IdentExpr
                "Item"))
            " "
            (BlockElementDeclarationBody
              "{"
              "}"))
          " "
          "}"))
      " "
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn get_with_body() {
  let tree = parse_source(
    r#"type T { namespace { get count(): int {
  0
} } }"#,
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "T")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          "namespace")
        " "
        (BlockElementDeclarationBody
          "{"
          " "
          (GetDeclaration
            "get"
            " "
            (GetDeclarationName
              "count")
            (FuncDeclarationParams
              "("
              ")")
            (FuncDeclarationReturnTyp
              ":"
              " "
              (IdentExpr
                "int"))
            " "
            (BlockElementDeclarationBody
              "{"
              "\n"
              "  "
              (ElementFieldDeclaration
                (ElementFieldDeclarationArg
                  (NumberExpr
                    "0")))
              "\n"
              "}"))
          " "
          "}"))
      " "
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn get_error_missing_body() {
  let tree = parse_source("type T { namespace { get count(): int } }");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "T")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          "namespace")
        " "
        (BlockElementDeclarationBody
          "{"
          " "
          (GetDeclaration
            "get"
            " "
            (GetDeclarationName
              "count")
            (FuncDeclarationParams
              "("
              ")")
            (FuncDeclarationReturnTyp
              ":"
              " "
              (IdentExpr
                "int")))
          " "
          "}"))
      " "
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn get_error_missing_name() {
  let tree = parse_source("type T { namespace { get (): int {} } }");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "T")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          "namespace")
        " "
        (BlockElementDeclarationBody
          "{"
          " "
          (GetDeclaration
            "get"
            " "
            (GetDeclarationName)
            (FuncDeclarationParams
              "("
              ")")
            (FuncDeclarationReturnTyp
              ":"
              " "
              (IdentExpr
                "int"))
            " "
            (BlockElementDeclarationBody
              "{"
              "}"))
          " "
          "}"))
      " "
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn get_no_params_no_return_type() {
  let tree = parse_source("type T { namespace { get items() {} } }");
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      "type")
    " "
    (EqualityDeclarationName
      "T")
    " "
    (BlockElementDeclarationBody
      "{"
      " "
      (BlockElementDeclaration
        (ElementDeclarationTyp
          "namespace")
        " "
        (BlockElementDeclarationBody
          "{"
          " "
          (GetDeclaration
            "get"
            " "
            (GetDeclarationName
              "items")
            (FuncDeclarationParams
              "("
              ")")
            " "
            (BlockElementDeclarationBody
              "{"
              "}"))
          " "
          "}"))
      " "
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}
