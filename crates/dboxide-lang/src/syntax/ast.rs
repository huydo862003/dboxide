#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
  // TODO: Define syntax kinds for syntax nodes

  // Shared tokens
  Ident = 400,
  DqString,
  SqString,
  OString,
  Number,
  Colon,        // :
  Comma,        // ,
  LParen,       // (
  RParen,       // )
  LBracket,     // [
  RBracket,     // ]
  LBrace,       // {
  RBrace,       // }
  Operator,

  // Trivia
  Whitespace = 600,
  Newline,
  Eof,

  // Error
  Error,
}
