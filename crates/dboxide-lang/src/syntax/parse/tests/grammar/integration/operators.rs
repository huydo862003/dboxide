use crate::syntax::parse::tests::utils::*;

#[test]
fn mul_before_add_precedence() {
  // a * b + c = (a * b) + c  (not a * (b + c))
  let tree = parse_expr("a * b + c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "*"
    (IdentExpr
      " "
      "b"))
  " "
  "+"
  (IdentExpr
    " "
    "c"))"#
  );
}

#[test]
fn add_before_mul_precedence() {
  // a + b * c = a + (b * c)  (not (a + b) * c)
  let tree = parse_expr("a + b * c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (IdentExpr
    "a")
  " "
  "+"
  (InfixExpr
    (IdentExpr
      " "
      "b")
    " "
    "*"
    (IdentExpr
      " "
      "c")))"#
  );
}

#[test]
fn and_before_or_precedence() {
  // a && b || c = (a && b) || c
  let tree = parse_expr("a && b || c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "&&"
    (IdentExpr
      " "
      "b"))
  " "
  "||"
  (IdentExpr
    " "
    "c"))"#
  );
}

#[test]
fn or_then_and_precedence() {
  // a || b && c = a || (b && c)
  let tree = parse_expr("a || b && c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (IdentExpr
    "a")
  " "
  "||"
  (InfixExpr
    (IdentExpr
      " "
      "b")
    " "
    "&&"
    (IdentExpr
      " "
      "c")))"#
  );
}

#[test]
fn subtype_before_and_precedence() {
  // a <: b && c = (a <: b) && c
  let tree = parse_expr("a <: b && c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "<:"
    (IdentExpr
      " "
      "b"))
  " "
  "&&"
  (IdentExpr
    " "
    "c"))"#
  );
}

#[test]
fn pipe_operator_lower_than_and() {
  // a && b | c = (a && b) | c
  let tree = parse_expr("a && b | c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "&&"
    (IdentExpr
      " "
      "b"))
  " "
  "|"
  (IdentExpr
    " "
    "c"))"#
  );
}

#[test]
fn add_is_left_associative() {
  // a + b + c = (a + b) + c
  let tree = parse_expr("a + b + c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "+"
    (IdentExpr
      " "
      "b"))
  " "
  "+"
  (IdentExpr
    " "
    "c"))"#
  );
}

#[test]
fn and_is_left_associative() {
  // a && b && c = (a && b) && c
  let tree = parse_expr("a && b && c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "&&"
    (IdentExpr
      " "
      "b"))
  " "
  "&&"
  (IdentExpr
    " "
    "c"))"#
  );
}

#[test]
fn or_is_left_associative() {
  // a || b || c = (a || b) || c
  let tree = parse_expr("a || b || c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "||"
    (IdentExpr
      " "
      "b"))
  " "
  "||"
  (IdentExpr
    " "
    "c"))"#
  );
}

#[test]
fn nullcoalesce_is_right_associative() {
  // a ?? b ?? c = a ?? (b ?? c)
  let tree = parse_expr("a ?? b ?? c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (IdentExpr
    "a")
  " "
  "??"
  (InfixExpr
    (IdentExpr
      " "
      "b")
    " "
    "??"
    (IdentExpr
      " "
      "c")))"#
  );
}

#[test]
fn arrow_is_right_associative() {
  // a => b => c = a => (b => c)
  let tree = parse_expr("a => b => c");
  assert_eq!(
    tree,
    r#"(ClosureExpr
  (IdentExpr
    "a")
  " "
  "=>"
  (ClosureExpr
    (IdentExpr
      " "
      "b")
    " "
    "=>"
    (IdentExpr
      " "
      "c")))"#
  );
}

#[test]
fn subtype_operator() {
  let tree = parse_expr("a <: b");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (IdentExpr
    "a")
  " "
  "<:"
  (IdentExpr
    " "
    "b"))"#
  );
}

#[test]
fn logical_and_operator() {
  let tree = parse_expr("a && b");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (IdentExpr
    "a")
  " "
  "&&"
  (IdentExpr
    " "
    "b"))"#
  );
}

#[test]
fn logical_or_operator() {
  let tree = parse_expr("a || b");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (IdentExpr
    "a")
  " "
  "||"
  (IdentExpr
    " "
    "b"))"#
  );
}

#[test]
fn pipe_operator() {
  let tree = parse_expr("a | b");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (IdentExpr
    "a")
  " "
  "|"
  (IdentExpr
    " "
    "b"))"#
  );
}

#[test]
fn arrow_operator() {
  let tree = parse_expr("a => b");
  assert_eq!(
    tree,
    r#"(ClosureExpr
  (IdentExpr
    "a")
  " "
  "=>"
  (IdentExpr
    " "
    "b"))"#
  );
}

#[test]
fn nullcoalesce_operator() {
  let tree = parse_expr("a ?? b");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (IdentExpr
    "a")
  " "
  "??"
  (IdentExpr
    " "
    "b"))"#
  );
}

#[test]
fn or_lower_than_nullcoalesce() {
  // a || b ?? c = (a || b) ?? c
  let tree = parse_expr("a || b ?? c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "||"
    (IdentExpr
      " "
      "b"))
  " "
  "??"
  (IdentExpr
    " "
    "c"))"#
  );
}

#[test]
fn pipe_lower_than_nullcoalesce() {
  // a | b ?? c = (a | b) ?? c
  let tree = parse_expr("a | b ?? c");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "|"
    (IdentExpr
      " "
      "b"))
  " "
  "??"
  (IdentExpr
    " "
    "c"))"#
  );
}

#[test]
fn range_infix_when_followed_by_prefix_operator() {
  // a.. -1 = a..(-1) not (a..) followed by -1
  // `..` is infix when next token is a prefix operator like `-`
  let tree = parse_expr("a.. -1");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (IdentExpr
    "a")
  ".."
  (PrefixExpr
    " "
    "-"
    (NumberExpr
      "1")))"#
  );
}

#[test]
fn range_postfix_does_not_bind_inside_add_rhs() {
  // a + b.. = (a + b)..
  let tree = parse_expr("a + b..");
  assert_eq!(
    tree,
    r#"(PostfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "+"
    (IdentExpr
      " "
      "b"))
  "..")"#
  );
}

#[test]
fn mul_add_compare_and_or() {
  // a * b + c == d && e || f
  // = ((((a * b) + c) == d) && e) || f
  let tree = parse_expr("a * b + c == d && e || f");
  assert_eq!(
    tree,
    r#"(InfixExpr
  (InfixExpr
    (InfixExpr
      (InfixExpr
        (InfixExpr
          (IdentExpr
            "a")
          " "
          "*"
          (IdentExpr
            " "
            "b"))
        " "
        "+"
        (IdentExpr
          " "
          "c"))
      " "
      "=="
      (IdentExpr
        " "
        "d"))
    " "
    "&&"
    (IdentExpr
      " "
      "e"))
  " "
  "||"
  (IdentExpr
    " "
    "f"))"#
  );
}
