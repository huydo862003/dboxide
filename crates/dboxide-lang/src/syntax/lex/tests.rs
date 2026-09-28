use std::cell::RefCell;
use std::rc::Rc;

use crate::syntax::ast::SyntaxKind;
use crate::syntax::ast::cache::Cache;
use crate::syntax::lex::LexCtx;
use crate::types::PeekableStream;
use crate::types::diagnostics::Diagnostic;

fn lex(input: &str) -> (Vec<(SyntaxKind, String)>, Vec<Diagnostic>) {
  let cache = Rc::new(RefCell::new(Cache::new()));
  let stream: PeekableStream<'_, char> =
    itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
  let mut lexer = LexCtx::new(stream, cache);

  let mut tokens = Vec::new();
  let mut diags = Vec::new();
  loop {
    let result = lexer.lex();
    let kind = result.token.kind();
    let text = result.token.text().unwrap_or("").to_string();
    if let Some(diag) = result.diagnostic {
      diags.push(diag);
    }
    tokens.push((kind, text));
    if kind == SyntaxKind::Eof {
      break;
    }
  }
  (tokens, diags)
}

/* Basic (tokens, _) */

#[test]
fn basic_tokens() {
  let (tokens, _) = lex("Table users { id integer [pk] }");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "Table".into()));
  assert_eq!(tokens[2], (SyntaxKind::Ident, "users".into()));
  assert_eq!(tokens[4], (SyntaxKind::LBrace, "{".into()));
  assert_eq!(tokens[6], (SyntaxKind::Ident, "id".into()));
  assert_eq!(tokens[10], (SyntaxKind::LBracket, "[".into()));
  assert_eq!(tokens[11], (SyntaxKind::Ident, "pk".into()));
  assert_eq!(tokens[12], (SyntaxKind::RBracket, "]".into()));
  assert_eq!(tokens[14], (SyntaxKind::RBrace, "}".into()));
}

// Should not crash on empty input
#[test]
fn empty_input() {
  let (tokens, _) = lex("");
  assert_eq!(tokens, vec![(SyntaxKind::Eof, "".into())]);
}

// After the first Eof, the lexer returns Eof always
#[test]
fn eof_then_none() {
  let cache = Rc::new(RefCell::new(Cache::new()));
  let stream: PeekableStream<'_, char> =
    itertools::multipeek(Box::new("".chars()) as Box<dyn Iterator<Item = char> + '_>);
  let mut lexer = LexCtx::new(stream, cache);
  assert_eq!(lexer.lex().token.kind(), SyntaxKind::Eof);
  assert_eq!(lexer.lex().token.kind(), SyntaxKind::Eof);
  assert_eq!(lexer.lex().token.kind(), SyntaxKind::Eof);
  assert_eq!(lexer.lex().token.kind(), SyntaxKind::Eof);
}

/* Punctuation */

#[test]
fn all_punctuation() {
  let (tokens, _) = lex(":,()[]{}");
  assert_eq!(tokens[0], (SyntaxKind::Colon, ":".into()));
  assert_eq!(tokens[1], (SyntaxKind::Comma, ",".into()));
  assert_eq!(tokens[2], (SyntaxKind::LParen, "(".into()));
  assert_eq!(tokens[3], (SyntaxKind::RParen, ")".into()));
  assert_eq!(tokens[4], (SyntaxKind::LBracket, "[".into()));
  assert_eq!(tokens[5], (SyntaxKind::RBracket, "]".into()));
  assert_eq!(tokens[6], (SyntaxKind::LBrace, "{".into()));
  assert_eq!(tokens[7], (SyntaxKind::RBrace, "}".into()));
}

// Colon on its own is a punctuation
// Colon can be part of operators though
#[test]
fn colon_is_punctuation_not_operator() {
  let (tokens, _) = lex("key: value");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "key".into()));
  assert_eq!(tokens[1], (SyntaxKind::Colon, ":".into()));
}

/* Whitespace and newlines */

#[test]
fn whitespace_only() {
  let (tokens, _) = lex("   \t  ");
  assert_eq!(tokens[0], (SyntaxKind::Whitespace, "   \t  ".into()));
}

#[test]
fn consecutive_newlines() {
  let (tokens, _) = lex("\n\n\n");
  assert_eq!(tokens[0], (SyntaxKind::Newline, "\n".into()));
  assert_eq!(tokens[1], (SyntaxKind::Newline, "\n".into()));
  assert_eq!(tokens[2], (SyntaxKind::Newline, "\n".into()));
}

#[test]
fn newlines_separate_from_whitespace() {
  let (tokens, _) = lex("a\nb\n");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "a".into()));
  assert_eq!(tokens[1], (SyntaxKind::Newline, "\n".into()));
  assert_eq!(tokens[2], (SyntaxKind::Ident, "b".into()));
  assert_eq!(tokens[3], (SyntaxKind::Newline, "\n".into()));
}

/* Identifiers */

#[test]
fn ident_basic() {
  let (tokens, _) = lex("hello_world");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "hello_world".into()));
}

#[test]
fn ident_with_underscores_and_digits() {
  let (tokens, _) = lex("_foo bar_2 __init__ x123");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "_foo".into()));
  assert_eq!(tokens[2], (SyntaxKind::Ident, "bar_2".into()));
  assert_eq!(tokens[4], (SyntaxKind::Ident, "__init__".into()));
  assert_eq!(tokens[6], (SyntaxKind::Ident, "x123".into()));
}

/* Numbers */

#[test]
fn number_integer() {
  let (tokens, _) = lex("42");
  assert_eq!(tokens[0], (SyntaxKind::Number, "42".into()));
}

#[test]
fn number_float() {
  let (tokens, _) = lex("3.14");
  assert_eq!(tokens[0], (SyntaxKind::Number, "3.14".into()));
}

#[test]
fn number_scientific() {
  let (tokens, _) = lex("1.5e10");
  assert_eq!(tokens[0], (SyntaxKind::Number, "1.5e10".into()));
}

#[test]
fn number_scientific_negative_exponent() {
  let (tokens, _) = lex("2.5E-3");
  assert_eq!(tokens[0], (SyntaxKind::Number, "2.5E-3".into()));
}

#[test]
fn number_dot_ident_is_not_float() {
  let (tokens, _) = lex("3.foo");
  assert_eq!(tokens[0], (SyntaxKind::Number, "3".into()));
  assert_eq!(tokens[1], (SyntaxKind::Operator, ".".into()));
  assert_eq!(tokens[2], (SyntaxKind::Ident, "foo".into()));
}

#[test]
fn negative_number_is_operator_then_number() {
  let (tokens, _) = lex("-42");
  assert_eq!(tokens[0], (SyntaxKind::Operator, "-".into()));
  assert_eq!(tokens[1], (SyntaxKind::Number, "42".into()));
}

/* Operators */

#[test]
fn single_char_operators() {
  let (tokens, _) = lex(". @ | ? = < > ! - + * / % ^ & ~");
  let ops: Vec<_> = tokens
    .iter()
    .filter(|(k, _)| *k == SyntaxKind::Operator)
    .map(|(_, t)| t.as_str())
    .collect();
  assert_eq!(
    ops,
    vec![
      ".", "@", "|", "?", "=", "<", ">", "!", "-", "+", "*", "/", "%", "^", "&", "~"
    ]
  );
}

#[test]
fn multi_char_operators() {
  let (tokens, _) = lex("=> <: ?? ++ .. -> <- <> ?>? !!");
  let ops: Vec<_> = tokens
    .iter()
    .filter(|(k, _)| *k == SyntaxKind::Operator)
    .map(|(_, t)| t.as_str())
    .collect();
  assert_eq!(
    ops,
    vec!["=>", "<:", "??", "++", "..", "->", "<-", "<>", "?>?", "!!"]
  );
}

#[test]
fn operator_adjacent_to_ident() {
  let (tokens, _) = lex("a+b");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "a".into()));
  assert_eq!(tokens[1], (SyntaxKind::Operator, "+".into()));
  assert_eq!(tokens[2], (SyntaxKind::Ident, "b".into()));
}

#[test]
fn dot_between_idents() {
  let (tokens, _) = lex("users.id");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "users".into()));
  assert_eq!(tokens[1], (SyntaxKind::Operator, ".".into()));
  assert_eq!(tokens[2], (SyntaxKind::Ident, "id".into()));
}

#[test]
fn range_operator() {
  let (tokens, _) = lex("1..10");
  assert_eq!(tokens[0], (SyntaxKind::Number, "1".into()));
  assert_eq!(tokens[1], (SyntaxKind::Operator, "..".into()));
  assert_eq!(tokens[2], (SyntaxKind::Number, "10".into()));
}

#[test]
fn slash_alone_is_operator() {
  let (tokens, _) = lex("a / b");
  assert_eq!(tokens[2], (SyntaxKind::Operator, "/".into()));
}

/* Strings */

#[test]
fn dq_string() {
  let (tokens, _) = lex(r#""hello""#);
  assert_eq!(tokens[0], (SyntaxKind::DqString, r#""hello""#.into()));
}

#[test]
fn sq_string() {
  let (tokens, _) = lex("'world'");
  assert_eq!(tokens[0], (SyntaxKind::SqString, "'world'".into()));
}

#[test]
fn oq_string() {
  let (tokens, _) = lex("`now()`");
  assert_eq!(tokens[0], (SyntaxKind::OqString, "`now()`".into()));
}

#[test]
fn triple_sq_string() {
  let (tokens, _) = lex("'''line1\nline2'''");
  assert_eq!(tokens[0].0, SyntaxKind::TqString);
}

#[test]
fn empty_dq_string() {
  let (tokens, _) = lex(r#""""#);
  assert_eq!(tokens[0], (SyntaxKind::DqString, r#""""#.into()));
}

#[test]
fn empty_sq_string() {
  let (tokens, _) = lex("''");
  assert_eq!(tokens[0], (SyntaxKind::SqString, "''".into()));
}

#[test]
fn empty_oq_string() {
  let (tokens, _) = lex("``");
  assert_eq!(tokens[0], (SyntaxKind::OqString, "``".into()));
}

#[test]
fn dq_string_escape() {
  let (tokens, _) = lex(r#""hello \"world\"""#);
  assert_eq!(
    tokens[0],
    (SyntaxKind::DqString, r#""hello \"world\"""#.into())
  );
}

#[test]
fn sq_string_escape() {
  let (tokens, _) = lex(r"'it\'s'");
  assert_eq!(tokens[0].0, SyntaxKind::SqString);
}

#[test]
fn oq_string_multiline() {
  let (tokens, _) = lex("`select\n  *\nfrom t`");
  assert_eq!(tokens[0].0, SyntaxKind::OqString);
}

#[test]
fn oq_string_escape() {
  let (tokens, _) = lex(r"`select \`col\` from tbl`");
  assert_eq!(
    tokens[0],
    (SyntaxKind::OqString, r"`select \`col\` from tbl`".into())
  );
}

#[test]
fn oq_string_other_escape_not_treated_as_escape() {
  let (tokens, _) = lex(r"`select '\n' from \tbl`");
  assert_eq!(
    tokens[0],
    (SyntaxKind::OqString, r"`select '\n' from \tbl`".into())
  );
}

/* String errors */

#[test]
fn unterminated_dq_string() {
  let (tokens, diags) = lex("\"hello");
  assert_eq!(tokens[0].0, SyntaxKind::Error);
  assert_eq!(diags.len(), 1);
}

#[test]
fn unterminated_sq_string() {
  let (tokens, diags) = lex("'hello");
  assert_eq!(tokens[0].0, SyntaxKind::Error);
  assert_eq!(diags.len(), 1);
}

#[test]
fn unterminated_oq_string() {
  let (tokens, diags) = lex("`hello");
  assert_eq!(tokens[0].0, SyntaxKind::Error);
  assert_eq!(diags.len(), 1);
}

#[test]
fn unterminated_triple_sq_string() {
  let (tokens, diags) = lex("'''hello");
  assert_eq!(tokens[0].0, SyntaxKind::Error);
  assert_eq!(diags.len(), 1);
}

#[test]
fn dq_string_newline_terminates_with_error() {
  let (tokens, diags) = lex("\"hello\n");
  assert_eq!(tokens[0].0, SyntaxKind::Error);
  assert_eq!(diags.len(), 1);
}

/* Comments */

#[test]
fn line_comment() {
  let (tokens, _) = lex("a // comment\nb");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "a".into()));
  assert_eq!(tokens[2].0, SyntaxKind::LineComment);
  assert_eq!(tokens[3], (SyntaxKind::Newline, "\n".into()));
  assert_eq!(tokens[4], (SyntaxKind::Ident, "b".into()));
}

#[test]
fn block_comment() {
  let (tokens, _) = lex("a /* block */ b");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "a".into()));
  assert_eq!(tokens[2], (SyntaxKind::BlockComment, "/* block */".into()));
  assert_eq!(tokens[4], (SyntaxKind::Ident, "b".into()));
}

#[test]
fn unterminated_block_comment() {
  let (tokens, diags) = lex("/* unclosed");
  assert_eq!(tokens[0].0, SyntaxKind::Error);
  assert_eq!(diags.len(), 1);
}

#[test]
fn nested_block_comment_not_supported() {
  let (tokens, _) = lex("/* outer /* inner */ still */");
  assert_eq!(tokens[0].0, SyntaxKind::BlockComment);
  assert_eq!(tokens[2], (SyntaxKind::Ident, "still".into()));
}

/* Error recovery */

#[test]
fn unexpected_char() {
  let (tokens, diags) = lex("#");
  assert_eq!(tokens[0].0, SyntaxKind::Error);
  assert_eq!(diags.len(), 1);
}

/* Integration: DBML surface syntax */

#[test]
fn dbml_table() {
  let (tokens, _) = lex(
    r#"Table users [note: 'main'] {
  id integer [pk]
  name varchar [not null]
  created_at timestamp [default: `now()`]
}"#,
  );
  let idents: Vec<_> = tokens
    .iter()
    .filter(|(k, _)| *k == SyntaxKind::Ident)
    .map(|(_, t)| t.as_str())
    .collect();
  assert!(idents.contains(&"Table"));
  assert!(idents.contains(&"users"));
  assert!(idents.contains(&"id"));
  assert!(idents.contains(&"pk"));
  assert!(idents.contains(&"created_at"));

  let oq: Vec<_> = tokens
    .iter()
    .filter(|(k, _)| *k == SyntaxKind::OqString)
    .map(|(_, t)| t.as_str())
    .collect();
  assert_eq!(oq, vec!["`now()`"]);
}

#[test]
fn dbml_ref() {
  let (tokens, _) = lex("Ref: posts.user_id > users.id");
  assert_eq!(tokens[0], (SyntaxKind::Ident, "Ref".into()));
  assert_eq!(tokens[1], (SyntaxKind::Colon, ":".into()));
  assert_eq!(tokens[3], (SyntaxKind::Ident, "posts".into()));
  assert_eq!(tokens[4], (SyntaxKind::Operator, ".".into()));
  assert_eq!(tokens[5], (SyntaxKind::Ident, "user_id".into()));
  assert_eq!(tokens[7], (SyntaxKind::Operator, ">".into()));
}

#[test]
fn setting_list() {
  let (tokens, _) = lex("[pk, not null, default: 0]");
  assert_eq!(tokens[0], (SyntaxKind::LBracket, "[".into()));
  assert_eq!(tokens[1], (SyntaxKind::Ident, "pk".into()));
  assert_eq!(tokens[2], (SyntaxKind::Comma, ",".into()));
  assert!(tokens.iter().any(|(k, _)| *k == SyntaxKind::Colon));
  assert_eq!(tokens.last().unwrap().0, SyntaxKind::Eof);
}
