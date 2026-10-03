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
  #[message("invalid color literal: expected 3 or 6 hex digits, found {count}")]
  InvalidColorLitLength { count: usize },

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
  #[message("[unique] on field '{field_name}' requires [label], [label alias], or [setting]")]
  UniqueRequiresLabel { field_name: String },
  #[message(
    "a type can have at most one [label] field, but '{first}' and '{second}' both declare [label]"
  )]
  DuplicateLabelField { first: String, second: String },
  #[message(
    "a type can have at most one [label alias] field, but '{first}' and '{second}' both declare [label alias]"
  )]
  DuplicateLabelAliasField { first: String, second: String },
  #[message("[csv] on field '{field_name}' requires [arg: n] or [arg: n..]")]
  CsvRequiresArg { field_name: String },
  #[message(
    "field '{field_name}' is marked [attribute] in a [field] type and must have type 'bool' or 'bool?'"
  )]
  FieldAttributeMustBeBool { field_name: String },
  #[message(
    "field '{field_name}' has conflicting annotations: [{first}] and [{second}] cannot be combined"
  )]
  ConflictingFieldAnnotations {
    field_name: String,
    first: String,
    second: String,
  },
  #[message("{message}")]
  InvalidConstraintDefinition {
    constraint_name: String,
    message: String,
  },
  #[message(
    "nested element '{element_typ}' is not allowed inside a type body; only nested type definitions are permitted"
  )]
  DisallowedNestedElement { element_typ: String },
  #[message("unrecognized [use] value '{value}'; expected 'expand namespaces' or 'follow refs'")]
  UnrecognizedOnUseValue { value: String },
  #[message("a type cannot be both [element] and [field]")]
  ConflictingTypRoles,
  #[message(
    "nested type '{typ_name}' must declare a role: add [element] or [field] to its setting list"
  )]
  NestedTypMissingRole { typ_name: String },
  #[message("a type body may only declare one nested [field] type")]
  MultipleNestedFieldTyps,
  #[message(
    "unrecognized type setting '{setting}'; valid settings are 'element', 'field' and 'use'"
  )]
  UnrecognizedTypSetting { setting: String },
  #[message(
    "colon-syntax declaration '{field_name}: Type' is not valid in a type body; use 'name Type [annotations]' instead"
  )]
  ColonDeclarationInTypBody { field_name: String },
  #[message("namespace export '{field_name}' does not refer to a declared field")]
  UnknownNamespaceExport { field_name: String },
  #[message("a namespace block can only have one expanded field ('{field_name}' is a duplicate)")]
  DuplicateExpandedNamespaceField { field_name: String },
  #[message("expanded namespace field '{field_name}' must refer to a list-typed field")]
  ExpandedNamespaceFieldMustBeList { field_name: String },
  #[message("no element named '{name}' was found in the expanded namespace field")]
  MissingExpandedNamespaceMember { name: String },
  #[message("'{name}' is not a member of this element's namespace")]
  UnknownNamespaceMember { name: String },
  #[message(
    "colon-syntax '{field_name}: Type' is not valid in a namespace block; namespace blocks only support field re-exports and getters"
  )]
  ColonDeclarationInNamespaceBlock { field_name: String },
  #[message(
    "field '{field_name}' declares [arg] after a variadic [arg: n..], which captures all remaining positions"
  )]
  ArgAfterVariadicArg { field_name: String },
  #[message("getters cannot have parameters; remove the '(...)' to define a getter")]
  GetterWithParams,

  #[message("duplicate type declaration '{name}'")]
  DuplicateTypeDeclaration {
    name: String,
    first_start: usize,
    first_end: usize,
  },

  /* Posttype / Element Checking */
  #[message("duplicate field name '{name}' in element body")]
  DuplicateFieldDeclaration {
    name: String,
    first_start: usize,
    first_end: usize,
  },
  #[message("duplicate {typ_name} label '{name}'")]
  DuplicateElementLabel {
    name: String,
    typ_name: String,
    first_offset: usize,
  },
  #[message("element type '{name}' could not be resolved")]
  UnresolvedElementTyp { name: String },
  #[message(
    "comma-separated values are not allowed in field arg '{arg_name}' of '{typ_name}'; add [csv] to the field type to enable this syntax"
  )]
  CsvArgNotAllowed { arg_name: String, typ_name: String },
  #[message("unknown attribute '{name}' on element type '{typ_name}'")]
  UnknownAttribute { name: String, typ_name: String },
  #[message(
    "duplicate attribute '{name}' on element type '{typ_name}'; this attribute may only appear once"
  )]
  DuplicateAttribute { name: String, typ_name: String },
  #[message("unknown sub-element '{name}' on element type '{typ_name}'")]
  UnknownSubElement { name: String, typ_name: String },
  #[message("unknown setting '{name}' on element type '{typ_name}'")]
  UnknownSetting { name: String, typ_name: String },
  #[message(
    "duplicate setting '{name}' on element type '{typ_name}'; this setting may only appear once"
  )]
  DuplicateSetting { name: String, typ_name: String },
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

  /* Imports */
  #[message("symbol '{name}' not found in '{module_path}'")]
  UnresolvedImport { name: String, module_path: String },
  #[message("import specifier '{name}' requires a kind - use 'kind {name}' (e.g. 'type {name}')")]
  MissingImportKind { name: String },
  #[message(
    "import kind '{kind_name}' must have a [label] with [unique] to be used as a kind specifier"
  )]
  ImportKindMustBeUnique { kind_name: String },
  #[message("imported name '{name}' conflicts with a local declaration")]
  ImportConflictsWithLocal { name: String },
  #[message("imported name '{name}' conflicts with a previously imported symbol of the same name")]
  ConflictingImport { name: String },

  /* Resolve */
  #[message(
    "ambiguous name '{name}': multiple symbols match in this namespace; using the first candidate"
  )]
  AmbiguousChain { name: String },

  /* Typecheck */
  #[message("type mismatch: expected '{expected}', found '{actual}'")]
  TypeMismatch { expected: String, actual: String },
  #[message("expression has type 'never' and cannot produce a value")]
  NeverExpression,

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
