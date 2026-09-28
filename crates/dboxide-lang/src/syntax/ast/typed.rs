//! Layer 3: Typed AST nodes wrapping untyped RedNodes
//! Each AST type checks the SyntaxKind on cast, providing a type-safe API
//! over the generic tree structure

use std::hash::Hash;

pub use dboxide_macros::{AstNode, wrapper_ast_node};

use super::SyntaxKind;
use super::green::SyntaxToken;
use super::red::RedNode;

/// All AST nodes implement this trait
pub trait AstNode: Sized + Clone + Eq + Hash + Send + Sync {
  /// Try to cast a RedNode into this AST type
  /// Returns None if the SyntaxKind doesn't match
  fn cast(syntax: RedNode) -> Option<Self>;

  /// Access the underlying RedNode
  fn syntax(&self) -> &RedNode;
}

pub fn child<T: AstNode>(parent: &RedNode) -> Option<T> {
  parent.children().find_map(T::cast)
}

pub fn children<T: AstNode>(parent: &RedNode) -> impl Iterator<Item = T> {
  parent.children().filter_map(T::cast)
}

pub fn child_token(parent: &RedNode, kind: SyntaxKind) -> Option<SyntaxToken> {
  parent
    .children()
    .find(|c| c.kind() == kind)
    .and_then(|c| c.as_token())
}

pub fn child_token_text(parent: &RedNode, kind: SyntaxKind) -> Option<String> {
  child_token(parent, kind).and_then(|t| t.text().map(ToString::to_string))
}

/* Root */

/// Root of a DBML source file
#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct SourceFile(RedNode);

impl SourceFile {
  pub fn declarations(&self) -> impl Iterator<Item = ElementDeclaration> {
    children::<ElementDeclaration>(&self.0)
  }

  pub fn block_elements(&self) -> impl Iterator<Item = BlockElementDeclaration> {
    children::<BlockElementDeclaration>(&self.0)
  }

  pub fn inline_elements(&self) -> impl Iterator<Item = InlineElementDeclaration> {
    children::<InlineElementDeclaration>(&self.0)
  }

  pub fn use_declarations(&self) -> impl Iterator<Item = UseDeclaration> {
    children::<UseDeclaration>(&self.0)
  }
}

/* Element Declarations */

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct BlockElementDeclaration(RedNode);

impl BlockElementDeclaration {
  pub fn element_type(&self) -> Option<BlockElementDeclarationType> {
    child::<BlockElementDeclarationType>(&self.0)
  }

  pub fn element_target(&self) -> Option<BlockElementDeclarationTargetFragment> {
    child::<BlockElementDeclarationTargetFragment>(&self.0)
  }

  pub fn element_alias(&self) -> Option<BlockElementDeclarationAlias> {
    child::<BlockElementDeclarationAlias>(&self.0)
  }

  pub fn element_setting_list(&self) -> Option<SettingList> {
    child::<SettingList>(&self.0)
  }

  pub fn element_body(&self) -> Option<BlockElementDeclarationBody> {
    child::<BlockElementDeclarationBody>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct BlockElementDeclarationType(RedNode);

impl BlockElementDeclarationType {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct BlockElementDeclarationTargetFragment(RedNode);

impl BlockElementDeclarationTargetFragment {
  pub fn text(&self) -> String {
    self.0.text()
  }

  pub fn expr(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct BlockElementDeclarationAlias(RedNode);

impl BlockElementDeclarationAlias {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct BlockElementDeclarationBody(RedNode);

impl BlockElementDeclarationBody {
  pub fn element_fields(&self) -> impl Iterator<Item = ElementFieldDeclaration> {
    children::<ElementFieldDeclaration>(&self.0)
  }

  pub fn element_attributes(&self) -> impl Iterator<Item = ElementAttributeDeclaration> {
    children::<ElementAttributeDeclaration>(&self.0)
  }

  pub fn nested_elements(&self) -> impl Iterator<Item = BlockElementDeclaration> {
    children::<BlockElementDeclaration>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct InlineElementDeclaration(RedNode);

impl InlineElementDeclaration {
  pub fn element_type(&self) -> Option<BlockElementDeclarationType> {
    child::<BlockElementDeclarationType>(&self.0)
  }

  pub fn element_target(&self) -> Option<InlineElementDeclarationTarget> {
    child::<InlineElementDeclarationTarget>(&self.0)
  }

  pub fn element_alias(&self) -> Option<BlockElementDeclarationAlias> {
    child::<BlockElementDeclarationAlias>(&self.0)
  }

  pub fn element_setting_list(&self) -> Option<SettingList> {
    child::<SettingList>(&self.0)
  }

  pub fn element_body(&self) -> Option<InlineElementDeclarationBody> {
    child::<InlineElementDeclarationBody>(&self.0)
  }

  pub fn element_field(&self) -> Option<ElementFieldDeclaration> {
    child::<ElementFieldDeclaration>(&self.0)
  }

  pub fn nested_element(&self) -> Option<BlockElementDeclaration> {
    child::<BlockElementDeclaration>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct InlineElementDeclarationTarget(RedNode);

impl InlineElementDeclarationTarget {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct InlineElementDeclarationBody(RedNode);

impl InlineElementDeclarationBody {
  pub fn element_field(&self) -> Option<ElementFieldDeclaration> {
    child::<ElementFieldDeclaration>(&self.0)
  }
}

#[wrapper_ast_node(SyntaxKind = [BlockElementDeclaration, InlineElementDeclaration])]
pub struct ElementDeclaration(RedNode);

/* Field & Attribute Declarations */

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ElementFieldDeclaration(RedNode);

impl ElementFieldDeclaration {
  pub fn exprs(&self) -> impl Iterator<Item = Expr> {
    children::<Expr>(&self.0)
  }

  pub fn field_args(&self) -> impl Iterator<Item = ElementFieldDeclarationArg> {
    children::<ElementFieldDeclarationArg>(&self.0)
  }

  pub fn field_setting_list(&self) -> Option<SettingList> {
    child::<SettingList>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ElementFieldDeclarationArg(RedNode);

impl ElementFieldDeclarationArg {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ElementAttributeDeclaration(RedNode);

impl ElementAttributeDeclaration {
  pub fn name(&self) -> Option<ElementAttributeDeclarationName> {
    child::<ElementAttributeDeclarationName>(&self.0)
  }

  pub fn value(&self) -> Option<ElementAttributeDeclarationValue> {
    child::<ElementAttributeDeclarationValue>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ElementAttributeDeclarationName(RedNode);

impl ElementAttributeDeclarationName {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ElementAttributeDeclarationValue(RedNode);

impl ElementAttributeDeclarationValue {
  pub fn text(&self) -> String {
    self.0.text()
  }

  pub fn expr(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

/* Settings */

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct SettingList(RedNode);

impl SettingList {
  pub fn items(&self) -> impl Iterator<Item = SettingListItem> {
    children::<SettingListItem>(&self.0)
  }

  pub fn find_item(&self, name: &str) -> Option<SettingListItem> {
    self.items().find(|item| {
      item
        .name()
        .map(|n| n.text().trim() == name)
        .unwrap_or(false)
    })
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct SettingListItem(RedNode);

impl SettingListItem {
  pub fn name(&self) -> Option<SettingListItemName> {
    child::<SettingListItemName>(&self.0)
  }

  pub fn value(&self) -> Option<SettingListItemValue> {
    child::<SettingListItemValue>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct SettingListItemName(RedNode);

impl SettingListItemName {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct SettingListItemValue(RedNode);

impl SettingListItemValue {
  pub fn text(&self) -> String {
    self.0.text()
  }

  pub fn expr(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

/* Use Declarations */

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct UseDeclaration(RedNode);

impl UseDeclaration {
  pub fn is_reuse(&self) -> bool {
    self.0.text().trim_start().starts_with("reuse")
  }

  pub fn wildcard(&self) -> Option<Wildcard> {
    child::<Wildcard>(&self.0)
  }

  pub fn specifier_list(&self) -> Option<UseSpecifierList> {
    child::<UseSpecifierList>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct Wildcard(RedNode);

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct UseSpecifierList(RedNode);

impl UseSpecifierList {
  pub fn specifiers(&self) -> impl Iterator<Item = UseSpecifier> {
    children::<UseSpecifier>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct UseSpecifier(RedNode);

impl UseSpecifier {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

/* fn / get / type Declarations */

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct FnDeclaration(RedNode);

impl FnDeclaration {
  pub fn name(&self) -> Option<FnDeclarationName> {
    child::<FnDeclarationName>(&self.0)
  }

  pub fn params(&self) -> Option<FnDeclarationParams> {
    child::<FnDeclarationParams>(&self.0)
  }

  pub fn return_type(&self) -> Option<FnDeclarationReturnType> {
    child::<FnDeclarationReturnType>(&self.0)
  }

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    child::<BlockElementDeclarationBody>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct FnDeclarationName(RedNode);

impl FnDeclarationName {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct FnDeclarationParams(RedNode);

impl FnDeclarationParams {
  pub fn params(&self) -> impl Iterator<Item = FnDeclarationParam> {
    children::<FnDeclarationParam>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct FnDeclarationParam(RedNode);

impl FnDeclarationParam {
  pub fn name(&self) -> Option<Expr> {
    children::<Expr>(&self.0).next()
  }

  pub fn type_expr(&self) -> Option<Expr> {
    children::<Expr>(&self.0).nth(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct FnDeclarationReturnType(RedNode);

impl FnDeclarationReturnType {
  pub fn type_expr(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct GetDeclaration(RedNode);

impl GetDeclaration {
  pub fn name(&self) -> Option<GetDeclarationName> {
    child::<GetDeclarationName>(&self.0)
  }

  pub fn params(&self) -> Option<FnDeclarationParams> {
    child::<FnDeclarationParams>(&self.0)
  }

  pub fn return_type(&self) -> Option<FnDeclarationReturnType> {
    child::<FnDeclarationReturnType>(&self.0)
  }

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    child::<BlockElementDeclarationBody>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct GetDeclarationName(RedNode);

impl GetDeclarationName {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct TypeDeclaration(RedNode);

impl TypeDeclaration {
  pub fn name(&self) -> Option<TypeDeclarationName> {
    child::<TypeDeclarationName>(&self.0)
  }

  pub fn role(&self) -> Option<SettingList> {
    child::<SettingList>(&self.0)
  }

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    child::<BlockElementDeclarationBody>(&self.0)
  }

  pub fn alias_expr(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct TypeDeclarationName(RedNode);

impl TypeDeclarationName {
  pub fn text(&self) -> String {
    self.0.text()
  }
}

/* Expressions */

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct InfixExpr(RedNode);

impl InfixExpr {
  pub fn left(&self) -> Option<Expr> {
    children::<Expr>(&self.0).next()
  }

  pub fn right(&self) -> Option<Expr> {
    children::<Expr>(&self.0).nth(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct PrefixExpr(RedNode);

impl PrefixExpr {
  pub fn operand(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct PostfixExpr(RedNode);

impl PostfixExpr {
  pub fn operand(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct CallExpr(RedNode);

impl CallExpr {
  pub fn callee(&self) -> Option<Expr> {
    children::<Expr>(&self.0).next()
  }

  pub fn args(&self) -> impl Iterator<Item = Expr> {
    children::<Expr>(&self.0).skip(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct IndexExpr(RedNode);

impl IndexExpr {
  pub fn target(&self) -> Option<Expr> {
    children::<Expr>(&self.0).next()
  }

  pub fn indices(&self) -> impl Iterator<Item = Expr> {
    children::<Expr>(&self.0).skip(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ParenExpr(RedNode);

impl ParenExpr {
  pub fn inner(&self) -> Option<Expr> {
    child::<Expr>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ListExpr(RedNode);

impl ListExpr {
  pub fn items(&self) -> impl Iterator<Item = Expr> {
    children::<Expr>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct TupleExpr(RedNode);

impl TupleExpr {
  pub fn items(&self) -> impl Iterator<Item = Expr> {
    children::<Expr>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct NumberExpr(RedNode);

impl NumberExpr {
  /// Exact number token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    child_token(&self.0, SyntaxKind::Number)
  }

  /// Raw text of the number token
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|t| t.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Parse the number as f64
  pub fn value(&self) -> Option<f64> {
    self.raw_text().parse().ok()
  }

  /// Parse the number as i64
  pub fn int_value(&self) -> Option<i64> {
    self.raw_text().parse().ok()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct IdentExpr(RedNode);

impl IdentExpr {
  /// Exact identifier token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    child_token(&self.0, SyntaxKind::Ident)
  }

  /// Identifier name
  pub fn name(&self) -> String {
    self
      .token()
      .and_then(|t| t.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }
}

pub fn unescape_string(content: &str) -> String {
  unescaper::unescape(content).unwrap_or_else(|_| content.to_string())
}

/// Unescapes an open-quoted string where only backticks can be escaped with backslash
pub fn unescape_oq_string(content: &str) -> String {
  let mut result = String::with_capacity(content.len());
  let mut chars = content.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\\' && chars.peek() == Some(&'`') {
      result.push('`');
      chars.next();
    } else {
      result.push(c);
    }
  }
  result
}

pub fn normalize_multiline_indent(content: &str) -> String {
  let lines: Vec<&str> = content.split('\n').collect();
  let first_non_empty = lines.iter().position(|l| !l.trim_start().is_empty());
  let Some(start) = first_non_empty else {
    return content.to_string();
  };
  let trimmed_top = &lines[start..];
  let non_empty: Vec<&str> = trimmed_top
    .iter()
    .copied()
    .filter(|l| !l.trim_start().is_empty())
    .collect();
  if non_empty.is_empty() {
    return trimmed_top.join("\n");
  }
  let min_indent = non_empty
    .iter()
    .map(|l| l.len() - l.trim_start().len())
    .min()
    .unwrap_or(0);
  trimmed_top
    .iter()
    .map(|l| {
      if l.len() >= min_indent {
        &l[min_indent..]
      } else {
        l.trim_start()
      }
    })
    .collect::<Vec<_>>()
    .join("\n")
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct DqStringExpr(RedNode);

impl DqStringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    child_token(&self.0, SyntaxKind::DqString)
  }

  /// Raw text of the string token including double quotes
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|t| t.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing double quotes
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    let s = raw.strip_prefix('"').unwrap_or(&raw);
    s.strip_suffix('"').unwrap_or(s).to_string()
  }

  /// Unescaped string value
  pub fn value(&self) -> String {
    unescape_string(&self.content())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct SqStringExpr(RedNode);

impl SqStringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    child_token(&self.0, SyntaxKind::SqString)
  }

  /// Raw text of the string token including single quotes
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|t| t.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing single quotes
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    let s = raw.strip_prefix('\'').unwrap_or(&raw);
    s.strip_suffix('\'').unwrap_or(s).to_string()
  }

  /// Unescaped string value
  pub fn value(&self) -> String {
    unescape_string(&self.content())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct TqStringExpr(RedNode);

impl TqStringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    child_token(&self.0, SyntaxKind::TqString)
  }

  /// Raw text of the string token including triple single quotes
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|t| t.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing triple single quotes
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    let s = raw.strip_prefix("'''").unwrap_or(&raw);
    s.strip_suffix("'''").unwrap_or(s).to_string()
  }

  /// Unescaped string value normalized by dedenting common indentation
  pub fn value(&self) -> String {
    normalize_multiline_indent(&unescape_string(&self.content()))
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct OqStringExpr(RedNode);

impl OqStringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    child_token(&self.0, SyntaxKind::OqString)
  }

  /// Raw text of the string token including backticks
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|t| t.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing backticks
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    let s = raw.strip_prefix('`').unwrap_or(&raw);
    s.strip_suffix('`').unwrap_or(s).to_string()
  }

  /// Unescaped string value where only backticks are escaped
  pub fn value(&self) -> String {
    unescape_oq_string(&self.content())
  }
}

#[wrapper_ast_node(SyntaxKind = [DqStringExpr, SqStringExpr, TqStringExpr, OqStringExpr])]
pub struct StringExpr(RedNode);

impl StringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    match self.0.kind() {
      SyntaxKind::DqStringExpr => child_token(&self.0, SyntaxKind::DqString),
      SyntaxKind::SqStringExpr => child_token(&self.0, SyntaxKind::SqString),
      SyntaxKind::TqStringExpr => child_token(&self.0, SyntaxKind::TqString),
      SyntaxKind::OqStringExpr => child_token(&self.0, SyntaxKind::OqString),
      _ => None,
    }
  }

  /// Raw text of the string token including quotes
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|t| t.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing quotes
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    match self.0.kind() {
      SyntaxKind::TqStringExpr => {
        let s = raw.strip_prefix("'''").unwrap_or(&raw);
        s.strip_suffix("'''").unwrap_or(s).to_string()
      }
      SyntaxKind::DqStringExpr => {
        let s = raw.strip_prefix('"').unwrap_or(&raw);
        s.strip_suffix('"').unwrap_or(s).to_string()
      }
      SyntaxKind::SqStringExpr => {
        let s = raw.strip_prefix('\'').unwrap_or(&raw);
        s.strip_suffix('\'').unwrap_or(s).to_string()
      }
      SyntaxKind::OqStringExpr => {
        let s = raw.strip_prefix('`').unwrap_or(&raw);
        s.strip_suffix('`').unwrap_or(s).to_string()
      }
      _ => raw,
    }
  }

  /// Unescaped string value (normalized for multiline triple-quoted strings)
  pub fn value(&self) -> String {
    match self.0.kind() {
      SyntaxKind::TqStringExpr => {
        let unescaped = unescape_string(&self.content());
        normalize_multiline_indent(&unescaped)
      }
      SyntaxKind::OqStringExpr => unescape_oq_string(&self.content()),
      _ => unescape_string(&self.content()),
    }
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ClosureExpr(RedNode);

impl ClosureExpr {
  /// Parameter expression: either TupleExpr or IdentExpr
  pub fn params(&self) -> Option<Expr> {
    children::<Expr>(&self.0).next()
  }

  /// Body expression of the closure
  pub fn body(&self) -> Option<Expr> {
    children::<Expr>(&self.0).nth(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct CommaExpr(RedNode);

impl CommaExpr {
  /// Elements in the comma-separated list
  pub fn items(&self) -> impl Iterator<Item = Expr> {
    children::<Expr>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ForallExpr(RedNode);

impl ForallExpr {
  pub fn binding(&self) -> Option<Expr> {
    children::<Expr>(&self.0).next()
  }

  pub fn collection(&self) -> Option<Expr> {
    children::<Expr>(&self.0).nth(1)
  }

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    child::<BlockElementDeclarationBody>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
pub struct ExistsExpr(RedNode);

impl ExistsExpr {
  pub fn binding(&self) -> Option<Expr> {
    children::<Expr>(&self.0).next()
  }

  pub fn collection(&self) -> Option<Expr> {
    children::<Expr>(&self.0).nth(1)
  }

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    child::<BlockElementDeclarationBody>(&self.0)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, AstNode)]
#[ast_node(Error)]
pub struct ErrorNode(RedNode);

impl ErrorNode {
  /// Raw text of the error node
  pub fn text(&self) -> String {
    self.0.text()
  }
}

#[wrapper_ast_node(SyntaxKind = [
  InfixExpr,
  PrefixExpr,
  PostfixExpr,
  CallExpr,
  IndexExpr,
  ParenExpr,
  ListExpr,
  TupleExpr,
  NumberExpr,
  IdentExpr,
  DqStringExpr,
  SqStringExpr,
  TqStringExpr,
  OqStringExpr,
  ClosureExpr,
  CommaExpr,
  ForallExpr,
  ExistsExpr
])]
pub struct Expr(RedNode);

#[cfg(test)]
mod tests {
  use std::cell::RefCell;
  use std::rc::Rc;

  use super::*;
  use crate::ast::cache::Cache;
  use crate::syntax::parse::ParseCtx;
  use crate::types::PeekableStream;

  #[test]
  fn test_typed_ast_navigation() {
    let input = r#"Table users as u [note: 'user table'] {
  id integer [pk]
  note: 'user note'
  indexes {
    id
  }
}"#;
    let cache = Rc::new(RefCell::new(Cache::new()));
    let stream: PeekableStream<'_, char> =
      itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
    let ctx = ParseCtx::new(stream, cache);
    let res = ctx.parse();
    let root = RedNode::new_root(res.ast.as_node().unwrap().clone());
    let source_file = SourceFile::cast(root).expect("SourceFile cast");

    let table = source_file.block_elements().next().expect("Table element");
    assert_eq!(table.element_type().unwrap().text().trim(), "Table");
    assert_eq!(table.element_target().unwrap().text().trim(), "users");
    assert_eq!(table.element_alias().unwrap().text().trim(), "u");

    let settings = table.element_setting_list().expect("setting list");
    let note_item = settings.find_item("note").expect("find note item");
    assert_eq!(note_item.name().unwrap().text().trim(), "note");
    assert_eq!(note_item.value().unwrap().text().trim(), "'user table'");

    let body = table.element_body().expect("table body");
    let field = body.element_fields().next().expect("id field");
    let field_exprs: Vec<_> = field.exprs().map(|a| a.0.text()).collect();
    assert_eq!(field_exprs.len(), 2);
    assert_eq!(field_exprs[0].trim(), "id");
    assert_eq!(field_exprs[1].trim(), "integer");

    let attr = body.element_attributes().next().expect("note attribute");
    assert_eq!(attr.name().unwrap().text().trim(), "note");
    assert_eq!(attr.value().unwrap().text().trim(), "'user note'");

    let nested = body.nested_elements().next().expect("nested indexes");
    assert_eq!(nested.element_type().unwrap().text().trim(), "indexes");
  }

  #[test]
  fn test_string_and_number_extraction() {
    let dq = parse_expr_node(r#""hello\nworld\"""#);
    let dq_expr = DqStringExpr::cast(dq.clone()).expect("DqStringExpr");
    assert_eq!(dq_expr.content(), r#"hello\nworld\""#);
    assert_eq!(dq_expr.value(), "hello\nworld\"");
    assert!(StringExpr::cast(dq).is_some());

    let sq = parse_expr_node(r#"'tab\there\'s'"#);
    let sq_expr = SqStringExpr::cast(sq.clone()).expect("SqStringExpr");
    assert_eq!(sq_expr.content(), r#"tab\there\'s"#);
    assert_eq!(sq_expr.value(), "tab\there's");

    let tq = parse_expr_node("'''\n    line 1\n    line 2\n'''");
    let tq_expr = TqStringExpr::cast(tq.clone()).expect("TqStringExpr");
    assert_eq!(tq_expr.value(), "line 1\nline 2\n");
    assert_eq!(StringExpr::cast(tq).unwrap().value(), "line 1\nline 2\n");

    let oq = parse_expr_node(r"`select \`col\` from tbl where c = '\n'`");
    let oq_expr = OqStringExpr::cast(oq.clone()).expect("OqStringExpr");
    assert_eq!(oq_expr.content(), r"select \`col\` from tbl where c = '\n'");
    assert_eq!(oq_expr.value(), r"select `col` from tbl where c = '\n'");
    assert_eq!(
      StringExpr::cast(oq).unwrap().value(),
      r"select `col` from tbl where c = '\n'"
    );

    let num = parse_expr_node("42");
    let num_expr = NumberExpr::cast(num).expect("NumberExpr");
    assert_eq!(num_expr.value(), Some(42.0));
    assert_eq!(num_expr.int_value(), Some(42));

    let float_num = parse_expr_node("12.5");
    let float_expr = NumberExpr::cast(float_num).expect("NumberExpr float");
    assert_eq!(float_expr.value(), Some(12.5));
    assert_eq!(float_expr.int_value(), None);

    // Verify robustness against trivia inside or surrounding expressions
    let num_with_trivia = parse_expr_node("/* comment */ 100");
    let num_trivia_expr = NumberExpr::cast(num_with_trivia).expect("NumberExpr with trivia");
    assert_eq!(num_trivia_expr.raw_text(), "100");
    assert_eq!(num_trivia_expr.value(), Some(100.0));
    assert_eq!(num_trivia_expr.int_value(), Some(100));

    let str_with_trivia = parse_expr_node("/* note */ \"escaped\\ncontent\"");
    let str_trivia_expr = DqStringExpr::cast(str_with_trivia).expect("DqStringExpr with trivia");
    assert_eq!(str_trivia_expr.raw_text(), "\"escaped\\ncontent\"");
    assert_eq!(str_trivia_expr.value(), "escaped\ncontent");
  }

  fn parse_expr_node(input: &str) -> RedNode {
    let cache = Rc::new(RefCell::new(Cache::new()));
    let stream: PeekableStream<'_, char> =
      itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
    let mut ctx = ParseCtx::new(stream, cache);
    let (green, _) = ctx.expr();
    RedNode::new_root(green.as_node().unwrap().clone())
  }
}
