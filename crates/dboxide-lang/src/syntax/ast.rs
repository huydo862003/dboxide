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

// TIL:
// - Rowan uses an internal u16 (rowan::SyntaxKind) to distinguish various kinds of syntax nodes
// - The user-defined syntax kind is for user-convenience
// - We need to switch back-and-forth between the 2 syntax kinds (see the DboxideLang)
impl From<SyntaxKind> for rowan::SyntaxKind {
  fn from(kind: SyntaxKind) -> Self {
    rowan::SyntaxKind(kind as u16)
  }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum DboxideLang {}

impl rowan::Language for DboxideLang {
  type Kind = SyntaxKind;

  fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
    unsafe {
      std::mem::transmute(raw.0)
    }
  }

  fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
    kind.into()
  }
}
