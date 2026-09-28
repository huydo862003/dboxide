use super::utils::*;

#[test]
fn parse_number_literal() {
  let tree = parse_expr("1");
  let expected = r#"(NumberExpr
  "1")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_decimal_literal() {
  let tree = parse_expr("3.14");
  let expected = r#"(NumberExpr
  "3.14")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_ident_literal() {
  let tree = parse_expr("users");
  let expected = r#"(IdentExpr
  "users")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_double_quoted_string() {
  let tree = parse_expr(r#""hello world""#);
  let expected = r#"(DqStringExpr
  "\"hello world\"")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_single_quoted_string() {
  let tree = parse_expr("'hello'");
  let expected = r#"(SqStringExpr
  "'hello'")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_binary_addition() {
  let tree = parse_expr("1 + 2");
  let expected = r#"(InfixExpr
  (NumberExpr
    "1")
  " "
  "+"
  (NumberExpr
    " "
    "2"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_binary_precedence_mul_add() {
  let tree = parse_expr("1 + 2 * 3");
  let expected = r#"(InfixExpr
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
      "3")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_paren_expression_precedence() {
  let tree = parse_expr("(1 + 2) * 3");
  let expected = r#"(InfixExpr
  (ParenExpr
    "("
    (InfixExpr
      (NumberExpr
        "1")
      " "
      "+"
      (NumberExpr
        " "
        "2"))
    ")")
  " "
  "*"
  (NumberExpr
    " "
    "3"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_binary_left_associative_sub() {
  let tree = parse_expr("1 - 2 - 3");
  let expected = r#"(InfixExpr
  (InfixExpr
    (NumberExpr
      "1")
    " "
    "-"
    (NumberExpr
      " "
      "2"))
  " "
  "-"
  (NumberExpr
    " "
    "3"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_binary_right_associative_coalesce() {
  let tree = parse_expr("a ?? b ?? c");
  let expected = r#"(InfixExpr
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
      "c")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_prefix_negate() {
  let tree = parse_expr("-x");
  let expected = r#"(PrefixExpr
  "-"
  (IdentExpr
    "x"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_prefix_not() {
  let tree = parse_expr("!ok");
  let expected = r#"(PrefixExpr
  "!"
  (IdentExpr
    "ok"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_prefix_mixin() {
  let tree = parse_expr("~Timestamps");
  let expected = r#"(PrefixExpr
  "~"
  (IdentExpr
    "Timestamps"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_prefix_range() {
  let tree = parse_expr("..10");
  let expected = r#"(PrefixExpr
  ".."
  (NumberExpr
    "10"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_postfix_nullable() {
  let tree = parse_expr("varchar?");
  let expected = r#"(PostfixExpr
  (IdentExpr
    "varchar")
  "?")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_call_no_args() {
  let tree = parse_expr("now()");
  let expected = r#"(CallExpr
  (IdentExpr
    "now")
  "("
  ")")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_call_multiple_args() {
  let tree = parse_expr("max(1, 2)");
  let expected = r#"(CallExpr
  (IdentExpr
    "max")
  "("
  (NumberExpr
    "1")
  ","
  (NumberExpr
    " "
    "2")
  ")")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_index_expression() {
  let tree = parse_expr("users[0]");
  let expected = r#"(IndexExpr
  (IdentExpr
    "users")
  "["
  (NumberExpr
    "0")
  "]")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_index_expression_disallows_spaces() {
  let tree = parse_expr("users [0]");
  let expected = r#"(IdentExpr
  "users")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_chained_call_and_index() {
  let tree = parse_expr("get_users()[0]");
  let expected = r#"(IndexExpr
  (CallExpr
    (IdentExpr
      "get_users")
    "("
    ")")
  "["
  (NumberExpr
    "0")
  "]")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_empty_list() {
  let tree = parse_expr("[]");
  let expected = r#"(ListExpr
  "["
  "]")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_list_items() {
  let tree = parse_expr("[1, 2]");
  let expected = r#"(ListExpr
  "["
  (NumberExpr
    "1")
  ","
  (NumberExpr
    " "
    "2")
  "]")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_empty_tuple() {
  let tree = parse_expr("()");
  let expected = r#"(TupleExpr
  "("
  ")")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_tuple_items() {
  let tree = parse_expr("(1, 2)");
  let expected = r#"(TupleExpr
  "("
  (NumberExpr
    "1")
  ","
  (NumberExpr
    " "
    "2")
  ")")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_single_element_tuple() {
  let tree = parse_expr("(1,)");
  let expected = r#"(TupleExpr
  "("
  (NumberExpr
    "1")
  ","
  ")")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_member_access() {
  let tree = parse_expr("user.name");
  let expected = r#"(InfixExpr
  (IdentExpr
    "user")
  "."
  (IdentExpr
    "name"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_infix_range() {
  let tree = parse_expr("1 .. 10");
  let expected = r#"(InfixExpr
  (NumberExpr
    "1")
  " "
  ".."
  (NumberExpr
    " "
    "10"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_postfix_range_in_list() {
  let tree = parse_expr("[1..]");
  let expected = r#"(ListExpr
  "["
  (PostfixExpr
    (NumberExpr
      "1")
    "..")
  "]")"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_subtype_expression() {
  let tree = parse_expr("Admin <: User");
  let expected = r#"(InfixExpr
  (IdentExpr
    "Admin")
  " "
  "<:"
  (IdentExpr
    " "
    "User"))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_comparison_and_logical() {
  let tree = parse_expr("a == b && c != d");
  let expected = r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "a")
    " "
    "=="
    (IdentExpr
      " "
      "b"))
  " "
  "&&"
  (InfixExpr
    (IdentExpr
      " "
      "c")
    " "
    "!="
    (IdentExpr
      " "
      "d")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_closure_expression_ident() {
  let tree = parse_expr("x => x + 1");
  let expected = r#"(ClosureExpr
  (IdentExpr
    "x")
  " "
  "=>"
  (InfixExpr
    (IdentExpr
      " "
      "x")
    " "
    "+"
    (NumberExpr
      " "
      "1")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_closure_expression_params() {
  let tree = parse_expr("(x, y) => x + y");
  let expected = r#"(ClosureExpr
  (TupleExpr
    "("
    (IdentExpr
      "x")
    ","
    (IdentExpr
      " "
      "y")
    ")")
  " "
  "=>"
  (InfixExpr
    (IdentExpr
      " "
      "x")
    " "
    "+"
    (IdentExpr
      " "
      "y")))"#;
  assert_eq!(tree, expected);
}

#[test]
fn parse_ref_operators() {
  let tree = parse_expr("orders.user_id > users.id");
  let expected = r#"(InfixExpr
  (InfixExpr
    (IdentExpr
      "orders")
    "."
    (IdentExpr
      "user_id"))
  " "
  ">"
  (InfixExpr
    (IdentExpr
      " "
      "users")
    "."
    (IdentExpr
      "id")))"#;
  assert_eq!(tree, expected);
}
