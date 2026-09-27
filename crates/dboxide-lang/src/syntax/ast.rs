#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
  /* Nodes */
  SourceFile = 0,
  
  // Example <block-element>:
  // <type> <target>? (as <alias>)? <setting-list>? {
  //   (<element-field> | <element-attribute> | <block-element>)*
  // }
  BlockElementDeclaration,
  BlockElementDeclarationType,
  BlockElementDeclarationTarget,
  BlockElementDeclarationAlias,
  BlockElementDeclarationBody,

  // Example <inline-element>:
  // <type> <target>? (as <alias>)? <setting-list>?: <element-field>
  InlineElementDeclaration,
  InlineElementDeclarationTarget,
  InlineElementDeclarationBody,

  // Example <element-field>:
  // <arg>+ <setting-list>?
  ElementFieldDeclaration,
  ElementFieldDeclarationArg,

  // Example <element-attribute>:
  // <name>: <value>
  ElementAttributeDeclaration,
  ElementAttributeDeclarationName,
  ElementAttributeDeclarationValue,

  // Example <setting-list>:
  // [<name>(: <value>)?,*]
  SettingList,
  SettingListItem,
  SettingListItemName,
  SettingListItemValue,

  /* Expression nodes */
  InfixExpression,
  PrefixExpression,
  PostfixExpression,

  ParenExpression,
  IndexExpression,
  CallExpression,
  ClosureExpression,

  ListExpression,
  TupleExpression,
  NumberExpression,
  DqStringExpression,
  SqStringExpression,
  OStringExpression,
  IdentExpression,

  /* Tokens */
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

impl From<SyntaxKind> for rowan::SyntaxKind {
  fn from(kind: SyntaxKind) -> Self {
    rowan::SyntaxKind(kind as u16)
  }
}
