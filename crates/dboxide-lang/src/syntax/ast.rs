#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
  /* Nodes */
  SOURCE_FILE = 0,
  
  // Example <block-element>:
  // <type> <target>? (as <alias>)? <setting-list>? {
  //   (<element-field> | <element-attribute> | <block-element>)*
  // }
  BLOCK_ELEMENT_DECLARATION,
  BLOCK_ELEMENT_DECLARATION_TYPE,
  BLOCK_ELEMENT_DECLARATION_TARGET,
  BLOCK_ELEMENT_DECLARATION_ALIAS,
  BLOCK_ELEMENT_DECLARATION_BODY,

  // Example <inline-element>:
  // <type> <target>? (as <alias>)? <setting-list>?: <element-field>
  INLINE_ELEMENT_DECLARATION,
  INLINE_ELEMENT_DECLARATION_TARGET,
  INLINE_ELEMENT_DECLARATION_BODY,

  // Example <element-field>:
  // <arg>+ <setting-list>?
  ELEMENT_FIELD_DECLARATION,
  ELEMENT_FIELD_DECLARATION_ARG,

  // Example <element-attribute>:
  // <name>: <value>
  ELEMENT_ATTRIBUTE_DECLARATION,
  ELEMENT_ATTRIBUTE_DECLARATION_NAME,
  ELEMENT_ATTRIBUTE_DECLARATION_VALUE,

  // Example <setting-list>:
  // [<name>(: <value>)?,*]
  SETTING_LIST,
  SETTING_LIST_ITEM,
  SETTING_LIST_ITEM_NAME,
  SETTING_LIST_ITEM_VALUE,

  /* Expression nodes */
  INFIX_EXPRESSION,
  PREFIX_EXPRESSION,
  POSTFIX_EXPRESSION,

  PAREN_EXPRESSION,
  INDEX_EXPRESSION,
  CALL_EXPRESSION,
  CLOSURE_EXPRESSION,

  LIST_EXPRESSION,
  TUPLE_EXPRESSION,
  NUMBER_EXPRESSION,
  DQ_STRING_EXPRESSION,
  SQ_STRING_EXPRESSION,
  OQ_STRING_EXPRESSION,
  IDENT_EXPRESSION,

  /* Tokens */
  IDENT = 400,
  DQ_STRING,
  SQ_STRING,
  OQ_STRING,
  NUMBER,
  COLON,         // :
  COMMA,         // ,
  L_PAREN,       // (
  R_PAREN,       // )
  L_BRACKET,     // [
  R_BRACKET,     // ]
  L_BRACE,       // {
  R_BRACE,       // }
  OPERATOR,

  // Trivia
  WHITESPACE = 600,
  NEWLINE,
  EOF,

  // Error
  ERROR,
}
pub use SyntaxKind::*;

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
