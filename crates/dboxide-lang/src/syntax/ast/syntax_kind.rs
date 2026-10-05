#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
  /* Nodes */
  SourceFile = 0,

  // <type> <target>? (as <alias>)? <setting-list>? {
  //   (<element-field> | <element-attribute> | <block-element>)*
  // }
  BlockElementDeclaration,
  BlockElementDeclarationBody,

  // <type> <target>? (as <alias>)? <setting-list>?: <element-field>
  InlineElementDeclaration,
  InlineElementDeclarationBody,

  ElementDeclarationTyp,
  ElementDeclarationTargetFragment,
  ElementDeclarationAlias,

  // <arg>+ <setting-list>?
  ElementFieldDeclaration,
  ElementFieldDeclarationArg,

  // <name>: <value>
  ElementAttributeDeclaration,
  ElementAttributeDeclarationName,
  ElementAttributeDeclarationValue,

  // [<name>(: <value>)?,*]
  SettingList,
  SettingListItem,
  SettingListItemName,
  SettingListItemValue,

  // use { ... } from '...' | use * from '...'
  UseDeclaration,
  UseSpecifierList,
  UseSpecifier,
  UseSpecifierKind,
  UseSpecifierName,
  UseSpecifierAlias,
  Wildcard,

  // fn name(params): ReturnType { body }
  // fn operator<sym>(params): ReturnType { body }
  FuncDeclaration,
  FuncDeclarationName,
  FuncDeclarationParams,
  FuncDeclarationParam,
  FuncDeclarationReturnTyp,

  // get name(params): ReturnType { body }
  GetDeclaration,
  GetDeclarationName,

  // type Name = expr
  EqualityDeclaration,
  EqualityDeclarationName,

  /* Expression nodes */
  CommaExpr,
  InfixExpr,
  PrefixExpr,
  PostfixExpr,

  ParenExpr,
  IndexExpr,
  CallExpr,
  ClosureExpr,

  ListExpr,
  TupleExpr,
  NumberExpr,
  DqStringExpr,
  SqStringExpr,
  TqStringExpr,
  OqStringExpr,
  IdentExpr,

  // forall <binding> of <collection> { body }
  // exists <binding> of <collection> { body }
  ForallExpr,
  ExistsExpr,

  /* Tokens */
  Ident = 400,
  DqString,
  SqString,
  TqString, // triple-quoted '''...'''
  OqString,
  Number,
  Colon,    // :
  Comma,    // ,
  LParen,   // (
  RParen,   // )
  LBracket, // [
  RBracket, // ]
  LBrace,   // {
  RBrace,   // }
  Operator,

  // Trivia
  Whitespace = 600,
  Newline,
  LineComment,
  BlockComment,
  Eof,

  // Error
  Error,
}

impl SyntaxKind {
  pub fn is_trivia(self) -> bool {
    matches!(
      self,
      Self::Whitespace | Self::Newline | Self::LineComment | Self::BlockComment
    )
  }
}
