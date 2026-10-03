use crate::syntax::parse::tests::utils::*;

mod declaration {
  use super::*;

  #[test]
  fn func_no_params_no_return() {
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
  fn func_one_param_with_return() {
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
  fn func_two_params() {
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
  fn func_no_params_no_parens() {
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

  #[test]
  fn func_no_parens_with_return_typ() {
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

  #[test]
  fn func_return_typ_union() {
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
}

mod operator {
  use super::*;

  #[test]
  fn func_operator_single_char() {
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
  fn func_operator_multi_char() {
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
  fn func_operator_with_space_before_symbol() {
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
}

mod body {
  use super::*;

  #[test]
  fn func_with_body_field() {
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
  fn func_inside_typ_body() {
    let tree = parse_source(
      r#"type T {
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
        "T"))
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
}

mod quantifiers {
  use super::*;

  #[test]
  fn func_forall_in_body() {
    let tree = parse_source(
      r#"fn check(): bool {
  forall c of columns { true }
}"#,
    );
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
  fn func_forall_dotted_collection() {
    let tree = parse_source(
      r#"fn check(): bool {
  forall c of table.columns { true }
}"#,
    );
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

  #[test]
  fn func_exists_in_body() {
    let tree = parse_source(
      r#"fn check(): bool {
  exists c of columns { true }
}"#,
    );
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
  fn func_exists_dotted_collection() {
    let tree = parse_source(
      r#"fn check(): bool {
  exists c of table.columns { true }
}"#,
    );
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

  #[test]
  fn func_two_quantifiers_in_body() {
    let tree = parse_source(
      r#"fn check(): bool {
  forall a of xs { true }
  exists b of ys { true }
}"#,
    );
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
}

mod error_recovery {
  use super::*;

  #[test]
  fn func_error_missing_name() {
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
  fn func_error_missing_body() {
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
  fn func_error_operator_missing_symbol() {
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
      (IdentExpr
        "foo")))
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
  fn func_error_param_missing_colon() {
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
  fn func_error_unclosed_params() {
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
  fn func_error_bad_token_in_params() {
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
  fn func_error_forall_missing_of() {
    let tree = parse_source(
      r#"fn check(): bool {
  forall c columns { true }
}"#,
    );
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
  fn func_error_forall_missing_body() {
    let tree = parse_source(
      r#"fn check(): bool {
  forall c of columns
}"#,
    );
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

  #[test]
  fn func_error_only_keyword() {
    let tree = parse_source("fn");
    let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    (FuncDeclarationName))
  "")"#;
    assert_eq!(tree, expected);
  }

  #[test]
  fn func_error_param_no_colon_or_typ() {
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

  #[test]
  fn func_error_forall_missing_binding() {
    // "of" is consumed as the binding IdentExpr; "columns" becomes the collection.
    // The parser emits a diagnostic for the missing "of" keyword and recovers.
    let tree = parse_source(
      r#"fn check(): bool {
  forall of columns { true }
}"#,
    );
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
}
