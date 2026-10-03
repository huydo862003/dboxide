use crate::syntax::parse::tests::utils::*;
use crate::types::diagnostics::Diagnostic;

// ---- fn declaration diagnostics ----

#[test]
fn fn_missing_name_diag() {
  let (tree, diags) = parse_source_with_diagnostics("fn {}");
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "function name or 'operator<sym>'",
      start_offset: 3,
      end_offset: 4,
    }]
  );
  let expected = r#"(SourceFile
  (FnDeclaration
    "fn"
    " "
    (FnDeclarationName)
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_missing_body_diag() {
  let (tree, diags) = parse_source_with_diagnostics("fn foo()");
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "'{'",
      start_offset: 8,
      end_offset: 8,
    }]
  );
  let expected = r#"(SourceFile
  (FnDeclaration
    "fn"
    " "
    (FnDeclarationName
      "foo")
    (FnDeclarationParams
      "("
      ")"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_only_keyword_diag() {
  let (tree, diags) = parse_source_with_diagnostics("fn");
  assert_eq!(
    diags,
    vec![
      Diagnostic::MissingExpectedToken {
        expected: "function name or 'operator<sym>'",
        start_offset: 2,
        end_offset: 2,
      },
      Diagnostic::MissingExpectedToken {
        expected: "'{'",
        start_offset: 2,
        end_offset: 2,
      },
    ]
  );
  let expected = r#"(SourceFile
  (FnDeclaration
    "fn"
    (FnDeclarationName))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn fn_operator_missing_symbol_diag() {
  let (_tree, diags) = parse_source_with_diagnostics("fn operator foo() {}");
  assert_eq!(
    diags,
    vec![
      Diagnostic::MissingExpectedToken {
        expected: "operator symbol after 'operator'",
        start_offset: 12,
        end_offset: 15,
      },
      Diagnostic::MissingExpectedToken {
        expected: "'{'",
        start_offset: 12,
        end_offset: 15,
      },
      Diagnostic::MissingExpectedToken {
        expected: "'{' or ':'",
        start_offset: 15,
        end_offset: 16,
      },
      Diagnostic::UnexpectedToken {
        expected: "element declaration",
        start_offset: 15,
        end_offset: 16,
      },
    ]
  );
}

#[test]
fn fn_param_missing_colon_diag() {
  // "fn foo(x int)" - x and int are each parsed as a bare-name param
  let (_tree, diags) = parse_source_with_diagnostics("fn foo(x int): bool {}");
  assert_eq!(
    diags,
    vec![
      Diagnostic::MissingExpectedToken {
        expected: "':' after parameter name",
        start_offset: 9,
        end_offset: 12,
      },
      Diagnostic::MissingExpectedToken {
        expected: "':' after parameter name",
        start_offset: 12,
        end_offset: 13,
      },
    ]
  );
}

#[test]
fn fn_unclosed_params_diag() {
  let (_tree, diags) = parse_source_with_diagnostics("fn foo(x: int");
  assert_eq!(
    diags,
    vec![
      Diagnostic::UnclosedDelimiter {
        delimiter: "(",
        open_offset: 6,
      },
      Diagnostic::MissingExpectedToken {
        expected: "'{'",
        start_offset: 13,
        end_offset: 13,
      },
    ]
  );
}

#[test]
fn fn_bad_token_in_params_diag() {
  let (_tree, diags) = parse_source_with_diagnostics("fn foo(123): bool {}");
  assert_eq!(
    diags,
    vec![Diagnostic::UnexpectedToken {
      expected: "parameter name",
      start_offset: 7,
      end_offset: 10,
    }]
  );
}

// ---- quantifier diagnostics ----

#[test]
fn forall_missing_of_diag() {
  let input = "fn check(): bool {\n  forall c columns { true }\n}";
  let (_tree, diags) = parse_source_with_diagnostics(input);
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "'of'",
      start_offset: 30,
      end_offset: 37,
    }]
  );
}

#[test]
fn forall_missing_body_diag() {
  let input = "fn check(): bool {\n  forall c of columns\n}";
  let (_tree, diags) = parse_source_with_diagnostics(input);
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "'{'",
      start_offset: 41,
      end_offset: 42,
    }]
  );
}

#[test]
fn forall_missing_binding_diag() {
  // "of" is consumed as the binding IdentExpr; the second Ident "columns"
  // is not the "of" keyword, so a diagnostic fires and "columns" becomes the collection.
  let input = "fn check(): bool {\n  forall of columns { true }\n}";
  let (_tree, diags) = parse_source_with_diagnostics(input);
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "'of'",
      start_offset: 31,
      end_offset: 38,
    }]
  );
}

// ---- type declaration diagnostics ----

#[test]
fn type_missing_name_diag() {
  let (tree, diags) = parse_source_with_diagnostics("type {}");
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "type name",
      start_offset: 5,
      end_offset: 6,
    }]
  );
  let expected = r#"(SourceFile
  (BlockElementDeclaration
    "type"
    " "
    (EqualityDeclarationName)
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}

#[test]
fn type_missing_body_or_eq_diag() {
  let (tree, diags) = parse_source_with_diagnostics("type Foo");
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "'{' or '='",
      start_offset: 8,
      end_offset: 8,
    }]
  );
  let expected = r#"(SourceFile
  (EqualityDeclaration
    "type"
    " "
    (EqualityDeclarationName
      "Foo"))
  "")"#;
  assert_eq!(tree, expected);
}

// ---- get declaration diagnostics ----

#[test]
fn get_missing_name_diag() {
  let input = "type T { namespace { get (): int {} } }";
  let (_tree, diags) = parse_source_with_diagnostics(input);
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "getter name",
      start_offset: 25,
      end_offset: 26,
    }]
  );
}

#[test]
fn get_missing_body_diag() {
  let input = "type T { namespace { get count(): int } }";
  let (_tree, diags) = parse_source_with_diagnostics(input);
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "'{'",
      start_offset: 38,
      end_offset: 39,
    }]
  );
}

// ---- error recovery: parser continues after broken declarations ----

#[test]
fn fn_missing_name_then_valid_fn() {
  let (tree, diags) = parse_source_with_diagnostics("fn {}\nfn ok() {}");
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "function name or 'operator<sym>'",
      start_offset: 3,
      end_offset: 4,
    }]
  );
  let expected = r#"(SourceFile
  (FnDeclaration
    "fn"
    " "
    (FnDeclarationName)
    (BlockElementDeclarationBody
      "{"
      "}"))
  "\n"
  (FnDeclaration
    "fn"
    " "
    (FnDeclarationName
      "ok")
    (FnDeclarationParams
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
fn fn_missing_body_then_valid_fn() {
  let (tree, diags) = parse_source_with_diagnostics("fn foo()\nfn bar() {}");
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "'{'",
      start_offset: 9,
      end_offset: 11,
    }]
  );
  let expected = r#"(SourceFile
  (FnDeclaration
    "fn"
    " "
    (FnDeclarationName
      "foo")
    (FnDeclarationParams
      "("
      ")"))
  "\n"
  (FnDeclaration
    "fn"
    " "
    (FnDeclarationName
      "bar")
    (FnDeclarationParams
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
fn type_missing_body_then_fn() {
  let (tree, diags) = parse_source_with_diagnostics("type Foo\nfn ok() {}");
  assert_eq!(
    diags,
    vec![Diagnostic::MissingExpectedToken {
      expected: "'{' or '='",
      start_offset: 9,
      end_offset: 11,
    }]
  );
  let expected = r#"(SourceFile
  (EqualityDeclaration
    "type"
    " "
    (EqualityDeclarationName
      "Foo"))
  "\n"
  (FnDeclaration
    "fn"
    " "
    (FnDeclarationName
      "ok")
    (FnDeclarationParams
      "("
      ")")
    " "
    (BlockElementDeclarationBody
      "{"
      "}"))
  "")"#;
  assert_eq!(tree, expected);
}
