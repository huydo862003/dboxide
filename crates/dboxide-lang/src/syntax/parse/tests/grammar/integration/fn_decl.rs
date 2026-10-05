use crate::syntax::parse::tests::utils::*;

#[test]
fn fn_no_params_no_return() {
  let tree = parse_source("fn foo() {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    (FuncDeclarationParams
      "("
      ")")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_one_param_with_return() {
  let tree = parse_source("fn foo(x: int): bool {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    (FuncDeclarationParams
      "("
      (FuncDeclarationParam
        (IdentExpr
          "x")
        ":"
        " "
        (IdentExpr
          "int"))
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
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_two_params() {
  let tree = parse_source("fn add(a: int, b: int): int {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "add")
    (FuncDeclarationParams
      "("
      (FuncDeclarationParam
        (IdentExpr
          "a")
        ":"
        " "
        (IdentExpr
          "int"))
      ","
      " "
      (FuncDeclarationParam
        (IdentExpr
          "b")
        ":"
        " "
        (IdentExpr
          "int"))
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
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_operator_single_char() {
  let tree = parse_source("fn operator>(a: int, b: int): bool {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "operator"
      ">")
    (FuncDeclarationParams
      "("
      (FuncDeclarationParam
        (IdentExpr
          "a")
        ":"
        " "
        (IdentExpr
          "int"))
      ","
      " "
      (FuncDeclarationParam
        (IdentExpr
          "b")
        ":"
        " "
        (IdentExpr
          "int"))
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
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_operator_multi_char() {
  let tree = parse_source("fn operator~>(a: bool, b: bool): bool {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "operator"
      "~>")
    (FuncDeclarationParams
      "("
      (FuncDeclarationParam
        (IdentExpr
          "a")
        ":"
        " "
        (IdentExpr
          "bool"))
      ","
      " "
      (FuncDeclarationParam
        (IdentExpr
          "b")
        ":"
        " "
        (IdentExpr
          "bool"))
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
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_with_body_field() {
  let tree = parse_source(
    r#"fn double(x: int): int {
  x + x
}"#,
  );
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "double")
    (FuncDeclarationParams
      "("
      (FuncDeclarationParam
        (IdentExpr
          "x")
        ":"
        " "
        (IdentExpr
          "int"))
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
          (InfixExpr
            (IdentExpr
              "x")
            " "
            "+"
            (IdentExpr
              " "
              "x"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_inside_type_body() {
  let tree = parse_source(
    r#"type T {
  fn method(): bool {}
}"#,
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
fn fn_return_type_union() {
  let tree = parse_source("fn parse(): int | null {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "parse")
    (FuncDeclarationParams
      "("
      ")")
    (FuncDeclarationReturnTyp
      ":"
      " "
      (InfixExpr
        (IdentExpr
          "int")
        " "
        "|"
        (IdentExpr
          " "
          "null")))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

// ---- fn without parens ----

#[test]
fn fn_no_params_no_parens() {
  let tree = parse_source("fn foo {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

// ---- quantifier expressions in fn body ----

#[test]
fn fn_forall_in_body() {
  let tree = parse_source("fn check(): bool {\n  forall c of columns { true }\n}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "check")
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
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (ForallExpr
            "forall"
            " "
            (IdentExpr
              "c")
            " "
            "of"
            " "
            (IdentExpr
              "columns")
            " "
            (BlockElementDeclarationBody
              "{"
              " "
              (ElementFieldDeclaration
                (ElementFieldDeclarationArg
                  (IdentExpr
                    "true")))
              " "
              "}"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_exists_in_body() {
  let tree = parse_source("fn check(): bool {\n  exists c of columns { true }\n}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "check")
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
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (ExistsExpr
            "exists"
            " "
            (IdentExpr
              "c")
            " "
            "of"
            " "
            (IdentExpr
              "columns")
            " "
            (BlockElementDeclarationBody
              "{"
              " "
              (ElementFieldDeclaration
                (ElementFieldDeclarationArg
                  (IdentExpr
                    "true")))
              " "
              "}"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_forall_dotted_collection() {
  let tree = parse_source("fn check(): bool {\n  forall c of table.columns { true }\n}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "check")
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
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (ForallExpr
            "forall"
            " "
            (IdentExpr
              "c")
            " "
            "of"
            " "
            (InfixExpr
              (IdentExpr
                "table")
              "."
              (IdentExpr
                "columns"))
            " "
            (BlockElementDeclarationBody
              "{"
              " "
              (ElementFieldDeclaration
                (ElementFieldDeclarationArg
                  (IdentExpr
                    "true")))
              " "
              "}"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

// ---- error recovery ----

#[test]
fn fn_error_missing_name() {
  let tree = parse_source("fn {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName)
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_error_missing_body() {
  let tree = parse_source("fn foo()");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    (FuncDeclarationParams
      "("
      ")"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_error_operator_missing_symbol() {
  let tree = parse_source("fn operator foo() {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "operator"))
  " "
  (Error
    (ElementDeclarationTyp
      "foo"))
  (Error
    "("
    ")"
    " "
    "{"
    "}")
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_error_param_missing_colon() {
  let tree = parse_source("fn foo(x int): bool {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    (FuncDeclarationParams
      "("
      (FuncDeclarationParam
        (IdentExpr
          "x"))
      " "
      (FuncDeclarationParam
        (IdentExpr
          "int"))
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
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_error_unclosed_params() {
  let tree = parse_source("fn foo(x: int");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    (FuncDeclarationParams
      "("
      (FuncDeclarationParam
        (IdentExpr
          "x")
        ":"
        " "
        (IdentExpr
          "int"))))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_error_bad_token_in_params() {
  let tree = parse_source("fn foo(123): bool {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    (FuncDeclarationParams
      "("
      "123"
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
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_error_forall_missing_of() {
  let tree = parse_source("fn check(): bool {\n  forall c columns { true }\n}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "check")
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
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (ForallExpr
            "forall"
            " "
            (IdentExpr
              "c")
            " "
            (IdentExpr
              "columns")
            " "
            (BlockElementDeclarationBody
              "{"
              " "
              (ElementFieldDeclaration
                (ElementFieldDeclarationArg
                  (IdentExpr
                    "true")))
              " "
              "}"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_error_forall_missing_body() {
  let tree = parse_source("fn check(): bool {\n  forall c of columns\n}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "check")
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
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (ForallExpr
            "forall"
            " "
            (IdentExpr
              "c")
            " "
            "of"
            " "
            (IdentExpr
              "columns"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

// ---- fn with return type but no parens ----

#[test]
fn fn_no_parens_with_return_type() {
  let tree = parse_source("fn foo: bool {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    (FuncDeclarationReturnTyp
      ":"
      " "
      (IdentExpr
        "bool"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

// ---- exists with dotted collection ----

#[test]
fn fn_exists_dotted_collection() {
  let tree = parse_source("fn check(): bool {\n  exists c of table.columns { true }\n}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "check")
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
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (ExistsExpr
            "exists"
            " "
            (IdentExpr
              "c")
            " "
            "of"
            " "
            (InfixExpr
              (IdentExpr
                "table")
              "."
              (IdentExpr
                "columns"))
            " "
            (BlockElementDeclarationBody
              "{"
              " "
              (ElementFieldDeclaration
                (ElementFieldDeclarationArg
                  (IdentExpr
                    "true")))
              " "
              "}"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

// ---- multiple quantifiers in one body ----

#[test]
fn fn_two_quantifiers_in_body() {
  let tree =
    parse_source("fn check(): bool {\n  forall a of xs { true }\n  exists b of ys { true }\n}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "check")
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
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (ForallExpr
            "forall"
            " "
            (IdentExpr
              "a")
            " "
            "of"
            " "
            (IdentExpr
              "xs")
            " "
            (BlockElementDeclarationBody
              "{"
              " "
              (ElementFieldDeclaration
                (ElementFieldDeclarationArg
                  (IdentExpr
                    "true")))
              " "
              "}"))))
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (ExistsExpr
            "exists"
            " "
            (IdentExpr
              "b")
            " "
            "of"
            " "
            (IdentExpr
              "ys")
            " "
            (BlockElementDeclarationBody
              "{"
              " "
              (ElementFieldDeclaration
                (ElementFieldDeclarationArg
                  (IdentExpr
                    "true")))
              " "
              "}"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

// ---- operator with space before symbol ----

#[test]
fn fn_operator_with_space_before_symbol() {
  let tree = parse_source("fn operator >(a: int): bool {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "operator"
      " "
      ">")
    (FuncDeclarationParams
      "("
      (FuncDeclarationParam
        (IdentExpr
          "a")
        ":"
        " "
        (IdentExpr
          "int"))
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
  "")"#;
  assert_eq!(tree, expected);
}

// ---- error: bare fn keyword with no name or body ----

#[test]
fn fn_error_only_keyword() {
  let tree = parse_source("fn");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    (FuncDeclarationName))
  "")"#;
  assert_eq!(tree, expected);
}

// ---- error: param with name only, no colon or type ----

#[test]
fn fn_error_param_no_colon_or_type() {
  let tree = parse_source("fn foo(x): bool {}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    (FuncDeclarationParams
      "("
      (FuncDeclarationParam
        (IdentExpr
          "x"))
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
  "")"#;
  assert_eq!(tree, expected);
}

// ---- error: forall with missing binding name ----

#[test]
fn fn_error_forall_missing_binding() {
  // "of" is consumed as the binding IdentExpr; "columns" becomes the collection.
  // The parser emits a diagnostic for the missing "of" keyword and recovers.
  let tree = parse_source("fn check(): bool {\n  forall of columns { true }\n}");
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "check")
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
      "\n"
      "  "
      (ElementFieldDeclaration
        (ElementFieldDeclarationArg
          (ForallExpr
            "forall"
            " "
            (IdentExpr
              "of")
            " "
            (IdentExpr
              "columns")
            " "
            (BlockElementDeclarationBody
              "{"
              " "
              (ElementFieldDeclaration
                (ElementFieldDeclarationArg
                  (IdentExpr
                    "true")))
              " "
              "}"))))
      "\n"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}
