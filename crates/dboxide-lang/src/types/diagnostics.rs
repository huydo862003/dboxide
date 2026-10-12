use std::path::PathBuf;

use dboxide_macros::dboxide_diagnostic_kind;

use crate::syntax::ast::SyntaxKind;

#[dboxide_diagnostic_kind]
pub enum DiagnosticKind {
  /* Lexer */
  #[message("unexpected end of input, expected '{expected}'")]
  UnexpectedEof { expected: char },
  #[message("unexpected character '{ch}'")]
  UnexpectedChar { ch: char },
  #[message("unterminated string literal")]
  UnterminatedString,

  /* Parser */
  #[message(
    "expected use declaration, block-form element declaration or inline-form element declaration"
  )]
  InvalidTopLevelElement,
  #[message("unexpected token, expected {expected}")]
  UnexpectedToken { expected: &'static str },
  #[message("expected {expected}")]
  MissingExpectedToken { expected: &'static str },
  #[message("unclosed '{delimiter}'")]
  UnclosedDelimiter { delimiter: &'static str },
  #[message("missing {expected:?}")]
  MissingSyntaxNode { expected: SyntaxKind },

  /* File */
  #[message("file not found: {path:?}")]
  FileNotFound { path: PathBuf },

  /* Semantic / Type Evaluation */
  #[message("{message}")]
  InvalidDeclarationFieldLabel { field_name: String, message: String },
  #[message(
    "non-contiguous arg index for field '{field_name}': found {found_index}, expected {expected_index}"
  )]
  NonContiguousFieldArgIndex {
    field_name: String,
    found_index: usize,
    expected_index: usize,
  },
  #[message("field '{field_name}' type annotation does not resolve to a type")]
  InvalidFieldTyp { field_name: String },
  #[message("rest attribute `[name] Type` must have exactly one identifier inside the brackets")]
  MalformedRestAttribute,
  #[message("a type can only have one rest attribute declaration")]
  DuplicateRestAttribute,
  #[message("rest attribute type must be a map type")]
  RestAttributeMustBeMap,
  #[message("field '{field_name}' cannot have both [label] and [label alias]")]
  ConflictingLabelAnnotations { field_name: String },
  #[message("[unique] on field '{field_name}' requires [label] or [label alias]")]
  UniqueRequiresLabel { field_name: String },
  #[message("[csv] on field '{field_name}' requires [arg: n] or [arg: n..]")]
  CsvRequiresArg { field_name: String },
  #[message("{message}")]
  InvalidConstraintDefinition {
    constraint_name: String,
    message: String,
  },
  #[message(
    "nested element '{element_typ}' is not allowed inside a type body; only nested type definitions are permitted"
  )]
  DisallowedNestedElement { element_typ: String },
  #[message("unrecognized [on use] value '{value}'; expected 'expand'")]
  UnrecognizedOnUseValue { value: String },
  #[message("a type cannot be both [element] and [field]")]
  ConflictingTypRoles,
  #[message(
    "nested type '{typ_name}' must declare a role: add [element] or [field] to its setting list"
  )]
  NestedTypMissingRole { typ_name: String },
  #[message(
    "unrecognized type setting '{setting}'; valid settings are 'element', 'field' and 'on use'"
  )]
  UnrecognizedTypSetting { setting: String },
  #[message(
    "colon-syntax declaration '{field_name}: Type' is not valid in a type body; use 'name Type [annotations]' instead"
  )]
  ColonDeclarationInTypBody { field_name: String },
  #[message("namespace export '{field_name}' does not refer to a declared field")]
  UnknownNamespaceExport { field_name: String },
  #[message(
    "colon-syntax '{field_name}: Type' is not valid in a namespace block; namespace blocks only support field re-exports and getters"
  )]
  ColonDeclarationInNamespaceBlock { field_name: String },
  #[message(
    "field '{field_name}' declares [arg] after a variadic [arg: n..], which captures all remaining positions"
  )]
  ArgAfterVariadicArg { field_name: String },

  /* Posttype / Element Checking */
  #[message("duplicate {typ_name} label '{name}'")]
  DuplicateElementLabel {
    name: String,
    typ_name: String,
    first_offset: usize,
  },
  #[message("element type '{name}' could not be resolved")]
  UnresolvedElementTyp { name: String },
  #[message("unknown attribute '{name}' on element type '{typ_name}'")]
  UnknownAttribute { name: String, typ_name: String },
  #[message("unknown sub-element '{name}' on element type '{typ_name}'")]
  UnknownSubElement { name: String, typ_name: String },
  #[message("unknown setting '{name}' on element type '{typ_name}'")]
  UnknownSetting { name: String, typ_name: String },
  #[message("constraint '{constraint_name}' violated on element type '{typ_name}'")]
  ConstraintViolation {
    constraint_name: String,
    typ_name: String,
  },
  #[message(
    "field '{field_name}' is declared [label alias] but no [label] field exists in this type; an alias has nothing to refer to"
  )]
  AliasLabelWithoutPrimaryLabel { field_name: String },
  #[message(
    "element type '{typ_name}' does not support aliases; add a field with [label alias] to enable 'as <name>' syntax"
  )]
  AliasNotSupportedByTyp { typ_name: String },
  #[message(
    "element type '{typ_name}' does not support named declarations; add a field with [label] to enable naming"
  )]
  NameNotSupportedByTyp { typ_name: String },
  #[message("'as <name>' requires a primary name; give the element a name before the alias")]
  AliasMissingPrimaryName,

  /* Field line arg count */
  #[message(
    "field line for '{typ_name}' expects at least {min_expected} positional argument(s), found {found}"
  )]
  TooFewFieldLineArgs {
    typ_name: String,
    min_expected: usize,
    found: usize,
  },
  #[message(
    "field line for '{typ_name}' expects at most {max_expected} positional argument(s), found {found}"
  )]
  TooManyFieldLineArgs {
    typ_name: String,
    max_expected: usize,
    found: usize,
  },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
  Error,
  Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Diagnostic {
  pub kind: DiagnosticKind,
  pub severity: Severity,
  pub start_offset: usize,
  pub end_offset: usize,
}

impl Diagnostic {
  pub fn code_str(&self) -> &'static str {
    self.kind.code_str()
  }

  pub fn offsets(&self) -> (usize, usize) {
    (self.start_offset, self.end_offset)
  }

  pub fn message(&self) -> String {
    self.kind.message()
  }
}
