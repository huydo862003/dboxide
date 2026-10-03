use crate::syntax::parse::tests::utils::*;
use crate::types::diagnostics::DiagnosticKind;

#[test]
fn func_missing_name_diagnostic() {
  let (tree, diagnostics) = parse_source_with_diagnostics("fn {}");
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::MissingExpectedToken {
        expected: "function name or 'operator<sym>'"
      }
      .into_error(3, 4)
    ]
  );
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
fn func_missing_body_diagnostic() {
  let (tree, diagnostics) = parse_source_with_diagnostics("fn foo()");
  assert_eq!(
    diagnostics,
    vec![DiagnosticKind::MissingExpectedToken { expected: "'{'" }.into_error(8, 8)]
  );
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
fn func_only_keyword_diagnostic() {
  let (tree, diagnostics) = parse_source_with_diagnostics("fn");
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::MissingExpectedToken {
        expected: "function name or 'operator<sym>'"
      }
      .into_error(2, 2),
      DiagnosticKind::MissingExpectedToken { expected: "'{'" }.into_error(2, 2),
    ]
  );
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    (FuncDeclarationName))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn func_operator_missing_symbol_diagnostic() {
  let (_tree, diagnostics) = parse_source_with_diagnostics("fn operator foo() {}");
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::MissingExpectedToken {
        expected: "operator symbol after 'operator'"
      }
      .into_error(12, 15),
      DiagnosticKind::MissingExpectedToken { expected: "'{'" }.into_error(12, 15),
      DiagnosticKind::MissingExpectedToken {
        expected: "'{' or ':'"
      }
      .into_error(15, 16),
      DiagnosticKind::UnexpectedToken {
        expected: "element declaration"
      }
      .into_error(15, 16),
    ]
  );
}

#[test]
fn func_param_missing_colon_diagnostic() {
  // "fn foo(x int)" - x and int are each parsed as a bare-name param
  let (_tree, diagnostics) = parse_source_with_diagnostics("fn foo(x int): bool {}");
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::MissingExpectedToken {
        expected: "':' after parameter name"
      }
      .into_error(9, 12),
      DiagnosticKind::MissingExpectedToken {
        expected: "':' after parameter name"
      }
      .into_error(12, 13),
    ]
  );
}

#[test]
fn func_unclosed_params_diagnostic() {
  let (_tree, diagnostics) = parse_source_with_diagnostics("fn foo(x: int");
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::UnclosedDelimiter { delimiter: "(" }.into_error(6, 6),
      DiagnosticKind::MissingExpectedToken { expected: "'{'" }.into_error(13, 13),
    ]
  );
}

#[test]
fn func_bad_token_in_params_diagnostic() {
  let (_tree, diagnostics) = parse_source_with_diagnostics("fn foo(123): bool {}");
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::UnexpectedToken {
        expected: "parameter name"
      }
      .into_error(7, 10)
    ]
  );
}

#[test]
fn forall_missing_of_diagnostic() {
  let input = "fn check(): bool {\n  forall c columns { true }\n}";
  let (_tree, diagnostics) = parse_source_with_diagnostics(input);
  assert_eq!(
    diagnostics,
    vec![DiagnosticKind::MissingExpectedToken { expected: "'of'" }.into_error(30, 37)]
  );
}

#[test]
fn forall_missing_body_diagnostic() {
  let input = "fn check(): bool {\n  forall c of columns\n}";
  let (_tree, diagnostics) = parse_source_with_diagnostics(input);
  assert_eq!(
    diagnostics,
    vec![DiagnosticKind::MissingExpectedToken { expected: "'{'" }.into_error(41, 42)]
  );
}

#[test]
fn forall_missing_binding_diagnostic() {
  // "of" is consumed as the binding IdentExpr; the second Ident "columns"
  // is not the "of" keyword, so a diagnostic fires and "columns" becomes the collection.
  let input = "fn check(): bool {\n  forall of columns { true }\n}";
  let (_tree, diagnostics) = parse_source_with_diagnostics(input);
  assert_eq!(
    diagnostics,
    vec![DiagnosticKind::MissingExpectedToken { expected: "'of'" }.into_error(31, 38)]
  );
}

#[test]
fn type_missing_name_diagnostic() {
  let (tree, diagnostics) = parse_source_with_diagnostics("type {}");
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::MissingExpectedToken {
        expected: "type name"
      }
      .into_error(5, 6)
    ]
  );
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
fn type_missing_body_or_eq_diagnostic() {
  let (tree, diagnostics) = parse_source_with_diagnostics("type Foo");
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::MissingExpectedToken {
        expected: "'{' or '='"
      }
      .into_error(8, 8)
    ]
  );
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
fn get_missing_name_diagnostic() {
  let input = "type T { namespace { get (): int {} } }";
  let (_tree, diagnostics) = parse_source_with_diagnostics(input);
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::MissingExpectedToken {
        expected: "getter name"
      }
      .into_error(25, 26)
    ]
  );
}

#[test]
fn get_missing_body_diagnostic() {
  let input = "type T { namespace { get count(): int } }";
  let (_tree, diagnostics) = parse_source_with_diagnostics(input);
  assert_eq!(
    diagnostics,
    vec![DiagnosticKind::MissingExpectedToken { expected: "'{'" }.into_error(38, 39)]
  );
}

#[test]
fn func_missing_name_then_valid_func() {
  let (tree, diagnostics) = parse_source_with_diagnostics(
    r#"fn {}
fn ok() {}"#,
  );
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::MissingExpectedToken {
        expected: "function name or 'operator<sym>'"
      }
      .into_error(3, 4)
    ]
  );
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName)
    (BlockElementDeclarationBody
      "{"
      "}"))
  "\n"
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "ok")
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
fn func_missing_body_then_valid_func() {
  let (tree, diagnostics) = parse_source_with_diagnostics(
    r#"fn foo()
fn bar() {}"#,
  );
  assert_eq!(
    diagnostics,
    vec![DiagnosticKind::MissingExpectedToken { expected: "'{'" }.into_error(9, 11)]
  );
  let expected = r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "foo")
    (FuncDeclarationParams
      "("
      ")"))
  "\n"
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "bar")
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
fn typ_missing_body_then_func() {
  let (tree, diagnostics) = parse_source_with_diagnostics(
    r#"type Foo
fn ok() {}"#,
  );
  assert_eq!(
    diagnostics,
    vec![
      DiagnosticKind::MissingExpectedToken {
        expected: "'{' or '='"
      }
      .into_error(9, 11)
    ]
  );
  let expected = r#"(SourceFile
  (EqualityDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "type"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "Foo")))
  "\n"
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "ok")
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
fn element_alias_garbage_tokens_recover_at_brace() {
  let (tree, diagnostics) = parse_source_with_diagnostics(r#"Table users as 123 garbage {}"#);
  assert!(!diagnostics.is_empty());
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    "as"
    (Error
      " "
      "123"
      " "
      "garbage")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn element_alias_garbage_stops_at_setting_list() {
  let (tree, diagnostics) = parse_source_with_diagnostics(r#"Table users as 123 [pk] {}"#);
  assert!(!diagnostics.is_empty());
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    "as"
    (Error
      " "
      "123")
    " "
    (SettingList
      "["
      (SettingListItem
        (SettingListItemName
          "pk"))
      "]")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn element_alias_garbage_stops_at_newline() {
  let (tree, _diagnostics) = parse_source_with_diagnostics(
    r#"Table users as 123
Table other {}"#,
  );
  let expected = r#"(SourceFile
  (Error
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "users"))
    " "
    "as"
    (Error
      " "
      "123"))
  "\n"
  (BlockElementDeclaration
    (ElementDeclarationTyp
      (IdentExpr
        "Table"))
    " "
    (ElementDeclarationTargetFragment
      (IdentExpr
        "other"))
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn paren_expr_eof_inside() {
  // Unclosed `(` at EOF
  let (_tree, diagnostics) = parse_source_with_diagnostics(
    r#"T E {
  val (1, 2
}"#,
  );
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.kind,
    DiagnosticKind::UnclosedDelimiter { delimiter } if *delimiter == "("
  )));
}

#[test]
fn tuple_eof_inside() {
  // Unclosed tuple paren at EOF
  let (_tree, diagnostics) = parse_source_with_diagnostics(
    "T E {
  val (1, 2",
  );
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.kind,
    DiagnosticKind::UnclosedDelimiter { delimiter } if *delimiter == "("
  )));
}

#[test]
fn setting_list_unclosed_at_eof() {
  // `[pk` without closing `]` at EOF
  let (_tree, diagnostics) = parse_source_with_diagnostics(
    "T E {
  id int [pk",
  );
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.kind,
    DiagnosticKind::UnclosedDelimiter { delimiter } if *delimiter == "["
  )));
}

#[test]
fn setting_list_unclosed_at_outer_brace() {
  let (tree, diagnostics) = parse_source_with_diagnostics(
    r#"T E {
  id int [pk
}"#,
  );
  assert!(!diagnostics.is_empty());
  let expected = r#"(SourceFile
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
          "\n"))
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn block_body_unclosed_at_eof() {
  let (_tree, diagnostics) = parse_source_with_diagnostics("T E {");
  assert!(diagnostics.iter().any(|diagnostic| matches!(
    &diagnostic.kind,
    DiagnosticKind::UnclosedDelimiter { delimiter } if *delimiter == "{"
  )));
}

#[test]
fn nested_block_unclosed_exits_parent() {
  // Nested block missing closing `}` - should exit parent context
  let (_tree, diagnostics) = parse_source_with_diagnostics(
    r#"T Outer {
  indexes {
    id
}"#,
  );
  // Should emit diagnostic for unclosed inner or outer block
  assert!(!diagnostics.is_empty());
}

#[test]
fn func_multiple_params_with_comma() {
  let tree = parse_source("fn check(a: int, b: string, c: bool): bool {}");
  assert_eq!(
    tree,
    r#"(SourceFile
  (FuncDeclaration
    "fn"
    " "
    (FuncDeclarationName
      "check")
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
          "string"))
      ","
      " "
      (FuncDeclarationParam
        (IdentExpr
          "c")
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
  "")"#
  );
}
