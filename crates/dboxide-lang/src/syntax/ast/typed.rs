//! Typed AST nodes wrapping untyped RedNodes

use std::hash::Hash;

pub use dboxide_macros::{AstNode, wrapper_ast_node};

use crate::DEFAULT_SCHEMA;

use super::SyntaxKind;
use super::green::SyntaxToken;
use super::red::RedNode;
use super::utils::{interpret_string, strip_quotes};

/// All AST nodes implement this trait
pub trait AstNode: Sized + Clone + Eq + Hash + Send + Sync {
  /// Try to cast a RedNode into this AST type
  /// Returns None if the SyntaxKind doesn't match
  fn cast(syntax: RedNode) -> Option<Self>;

  /// Access the underlying RedNode
  fn syntax(&self) -> &RedNode;

  /// Access text of the node
  fn text(&self) -> String {
    self.syntax().text()
  }

  /// Access child node of certain type
  fn child<T: AstNode>(&self) -> Option<T> {
    self.syntax().children().find_map(T::cast)
  }

  /// Iterator over child nodes of type T
  fn children<T: AstNode>(&self) -> impl Iterator<Item = T> {
    self.syntax().children().filter_map(T::cast)
  }

  /// Access child token of the node of certain type
  fn child_token(&self, kind: SyntaxKind) -> Option<SyntaxToken> {
    self
      .syntax()
      .children()
      .find(|child| child.kind() == kind)
      .and_then(|child| child.as_token())
  }
}

/* Root */

/// Root of a DBML source file
#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct SourceFile(RedNode);

impl SourceFile {
  pub fn declarations(&self) -> impl Iterator<Item = Declaration> {
    self.children::<Declaration>()
  }

  pub fn element_declarations(&self) -> impl Iterator<Item = ElementDeclaration> {
    self.children::<ElementDeclaration>()
  }

  pub fn use_declarations(&self) -> impl Iterator<Item = UseDeclaration> {
    self.children::<UseDeclaration>()
  }

  pub fn equality_declarations(&self) -> impl Iterator<Item = EqualityDeclaration> {
    self.children::<EqualityDeclaration>()
  }

  pub fn func_declarations(&self) -> impl Iterator<Item = FuncDeclaration> {
    self.children::<FuncDeclaration>()
  }
}

#[wrapper_ast_node(SyntaxKind = [
  BlockElementDeclaration,
  InlineElementDeclaration,
  UseDeclaration,
  FuncDeclaration,
  GetDeclaration,
  EqualityDeclaration,
])]
pub struct Declaration(RedNode);

/* Element Declarations */

#[wrapper_ast_node(SyntaxKind = [BlockElementDeclaration, InlineElementDeclaration])]
pub struct ElementDeclaration(RedNode);

impl ElementDeclaration {
  pub fn typ(&self) -> Option<ElementDeclarationTyp> {
    self.child::<ElementDeclarationTyp>()
  }

  pub fn alias(&self) -> Option<ElementDeclarationAlias> {
    self.child::<ElementDeclarationAlias>()
  }

  pub fn setting_list(&self) -> Option<SettingList> {
    self.child::<SettingList>()
  }

  pub fn target_kind(&self) -> Option<ElementDeclarationTargetFragment> {
    let mut iter = self.children::<ElementDeclarationTargetFragment>();
    let first = iter.next()?;
    // kind is only present when two fragments exist (e.g. `kind name`)
    iter.next().map(|_| first)
  }

  pub fn target_name(&self) -> Option<ElementDeclarationTargetFragment> {
    let mut iter = self.children::<ElementDeclarationTargetFragment>();
    let first = iter.next()?;
    // when two fragments: second is name; when one: it is the name
    Some(iter.next().unwrap_or(first))
  }

  pub fn is_typ_declaration(&self) -> bool {
    let Some(typ) = self.typ() else {
      return false;
    };
    let schemas = typ.schema();
    // Unqualified or explicitly `public.type`
    (schemas.is_empty()
      || (schemas.len() == 1 && schemas[0].name().eq_ignore_ascii_case(DEFAULT_SCHEMA)))
      && typ
        .name()
        .is_some_and(|field| field.name().eq_ignore_ascii_case("type"))
  }

  pub fn is_nested_typ(&self) -> bool {
    self.is_typ_declaration()
      && self
        .setting_list()
        .is_some_and(|setting| setting.has_item("element") || setting.has_item("field"))
  }

  pub fn is_nested_element_typ(&self) -> bool {
    self.is_typ_declaration()
      && self
        .setting_list()
        .is_some_and(|setting| setting.has_item("element"))
  }

  pub fn is_nested_field_typ(&self) -> bool {
    self.is_typ_declaration()
      && self
        .setting_list()
        .is_some_and(|setting| setting.has_item("field"))
  }

  pub fn is_namespace_declaration(&self) -> bool {
    self
      .typ()
      .and_then(|typ| typ.name())
      .is_some_and(|name_node| name_node.name().eq_ignore_ascii_case("namespace"))
  }

  pub fn is_constraints_declaration(&self) -> bool {
    self
      .typ()
      .and_then(|typ| typ.name())
      .is_some_and(|name_node| name_node.name().eq_ignore_ascii_case("constraints"))
  }
}

/// A single segment of a dotted path, carrying both its string name and source node
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PathFragment {
  node: RedNode,
  name: String,
}

impl PathFragment {
  fn new(node: RedNode, name: String) -> Self {
    Self { node, name }
  }

  /// The string name of this fragment, case-preserved
  pub fn name(&self) -> &str {
    &self.name
  }

  /// The underlying syntax node (IdentExpr or DqStringExpr)
  pub fn node(&self) -> RedNode {
    self.node.clone()
  }
}

// Declaration type
#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ElementDeclarationTyp(RedNode);

impl ElementDeclarationTyp {
  pub fn expr(&self) -> Option<Expr> {
    self.child::<Expr>()
  }

  /// All path segments with their source nodes
  pub fn path(&self) -> Vec<PathFragment> {
    self
      .expr()
      .and_then(|expr| expr.as_path())
      .unwrap_or_default()
  }

  /// Schema prefix segments, empty for unqualified paths
  pub fn schema(&self) -> Vec<PathFragment> {
    let mut path = self.path();
    if path.len() > 1 {
      path.pop();
      path
    } else {
      vec![]
    }
  }

  /// The own (rightmost) name fragment
  pub fn name(&self) -> Option<PathFragment> {
    self.path().into_iter().last()
  }
}

// Target kind or target name
#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ElementDeclarationTargetFragment(RedNode);

impl ElementDeclarationTargetFragment {
  pub fn expr(&self) -> Option<Expr> {
    self.child::<Expr>()
  }

  /// All path segments with their source nodes
  pub fn path(&self) -> Vec<PathFragment> {
    self
      .expr()
      .and_then(|expr| expr.as_path())
      .unwrap_or_default()
  }

  pub fn schema(&self) -> Vec<PathFragment> {
    let mut path = self.path();
    if path.len() > 1 {
      path.pop();
      path
    } else {
      vec![]
    }
  }

  /// The own (rightmost) name fragment
  pub fn name(&self) -> Option<PathFragment> {
    self.path().into_iter().last()
  }
}

// Alias
#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ElementDeclarationAlias(RedNode);

impl ElementDeclarationAlias {
  pub fn name(&self) -> Option<NameExpr> {
    self.child::<NameExpr>()
  }
}

/** Block element **/
#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct BlockElementDeclaration(RedNode);

impl BlockElementDeclaration {
  /* Most other methods are provided by ElementDeclaration already */

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    self.child::<BlockElementDeclarationBody>()
  }
}

// Body
#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct BlockElementDeclarationBody(RedNode);

impl BlockElementDeclarationBody {
  pub fn items(&self) -> impl Iterator<Item = BlockBodyItem> {
    self.children::<BlockBodyItem>()
  }

  pub fn fields(&self) -> impl Iterator<Item = ElementFieldDeclaration> {
    self.children::<ElementFieldDeclaration>()
  }

  pub fn attributes(&self) -> impl Iterator<Item = ElementAttributeDeclaration> {
    self.children::<ElementAttributeDeclaration>()
  }

  pub fn element_declarations(&self) -> impl Iterator<Item = BlockElementDeclaration> {
    self.children::<BlockElementDeclaration>()
  }

  pub fn func_declarations(&self) -> impl Iterator<Item = FuncDeclaration> {
    self.children::<FuncDeclaration>()
  }

  pub fn get_declarations(&self) -> impl Iterator<Item = GetDeclaration> {
    self.children::<GetDeclaration>()
  }

  pub fn equality_declarations(&self) -> impl Iterator<Item = EqualityDeclaration> {
    self.children::<EqualityDeclaration>()
  }

  pub fn namespace_body(&self) -> Option<BlockElementDeclarationBody> {
    self.element_declarations().find_map(|nested| {
      let element = ElementDeclaration::cast(nested.syntax().clone())?;
      if element.is_namespace_declaration() {
        BlockElementDeclaration::cast(element.syntax().clone())?.body()
      } else {
        None
      }
    })
  }

  pub fn constraints(&self) -> Option<BlockElementDeclaration> {
    self.element_declarations().find(|nested| {
      ElementDeclaration::cast(nested.syntax().clone())
        .is_some_and(|element| element.is_constraints_declaration())
    })
  }

  pub fn typ_declarations(&self) -> impl Iterator<Item = BlockElementDeclaration> {
    self.element_declarations().filter(|nested| {
      ElementDeclaration::cast(nested.syntax().clone())
        .is_some_and(|element| element.is_typ_declaration())
    })
  }

  pub fn disallowed_elements(&self) -> impl Iterator<Item = BlockElementDeclaration> {
    self.element_declarations().filter(|nested| {
      let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
        return false;
      };
      !element.is_typ_declaration()
        && !element.is_namespace_declaration()
        && !element.is_constraints_declaration()
    })
  }
}

// All allowed children of block elements
#[wrapper_ast_node(SyntaxKind = [
  ElementFieldDeclaration,
  ElementAttributeDeclaration,
  BlockElementDeclaration,
  FuncDeclaration,
  GetDeclaration,
  EqualityDeclaration,
])]
pub struct BlockBodyItem(RedNode);

/** Inline element **/

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct InlineElementDeclaration(RedNode);

impl InlineElementDeclaration {
  /* Most other methods are provided by ElementDeclaration already */

  pub fn body(&self) -> Option<InlineElementDeclarationBody> {
    self.child::<InlineElementDeclarationBody>()
  }

  pub fn field(&self) -> Option<ElementFieldDeclaration> {
    self.child::<ElementFieldDeclaration>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct InlineElementDeclarationBody(RedNode);

impl InlineElementDeclarationBody {
  pub fn items(&self) -> impl Iterator<Item = BlockBodyItem> {
    self.children::<BlockBodyItem>()
  }

  pub fn field(&self) -> Option<ElementFieldDeclaration> {
    self.child::<ElementFieldDeclaration>()
  }
}

/** Field declarations **/

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ElementFieldDeclaration(RedNode);

impl ElementFieldDeclaration {
  pub fn args(&self) -> impl Iterator<Item = ElementFieldDeclarationArg> {
    self.children::<ElementFieldDeclarationArg>()
  }

  pub fn setting_list(&self) -> Option<SettingList> {
    self.child::<SettingList>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ElementFieldDeclarationArg(RedNode);

impl ElementFieldDeclarationArg {
  pub fn expr(&self) -> Option<Expr> {
    self.child::<Expr>()
  }
}

/** Attribute declarations **/

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ElementAttributeDeclaration(RedNode);

impl ElementAttributeDeclaration {
  pub fn name(&self) -> Option<ElementAttributeDeclarationName> {
    self.child::<ElementAttributeDeclarationName>()
  }

  pub fn value(&self) -> Option<ElementAttributeDeclarationValue> {
    self.child::<ElementAttributeDeclarationValue>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ElementAttributeDeclarationName(RedNode);

impl ElementAttributeDeclarationName {
  pub fn name(&self) -> Option<NameExpr> {
    self.child::<NameExpr>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ElementAttributeDeclarationValue(RedNode);

impl ElementAttributeDeclarationValue {
  pub fn expr(&self) -> Option<Expr> {
    self.child::<Expr>()
  }
}

/** Settings **/

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct SettingList(RedNode);

impl SettingList {
  pub fn items(&self) -> impl Iterator<Item = SettingListItem> {
    self.children::<SettingListItem>()
  }

  pub fn get_item(&self, name: &str) -> Option<SettingListItem> {
    self.items().find(|item| {
      item
        .name()
        .is_some_and(|name_node| name_node.name().eq_ignore_ascii_case(name))
    })
  }

  pub fn has_item(&self, name: &str) -> bool {
    self.get_item(name).is_some()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct SettingListItem(RedNode);

impl SettingListItem {
  pub fn name(&self) -> Option<SettingListItemName> {
    self.child::<SettingListItemName>()
  }

  pub fn value(&self) -> Option<SettingListItemValue> {
    self.child::<SettingListItemValue>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct SettingListItemName(RedNode);

impl SettingListItemName {
  pub fn name(&self) -> String {
    strip_quotes(&self.text())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct SettingListItemValue(RedNode);

impl SettingListItemValue {
  pub fn expr(&self) -> Option<Expr> {
    self.child::<Expr>()
  }

  pub fn as_str(&self) -> String {
    strip_quotes(&self.text())
  }
}

/* Use declarations */

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct UseDeclaration(RedNode);

impl UseDeclaration {
  pub fn is_reuse(&self) -> bool {
    self.child_token(SyntaxKind::Ident).is_some_and(|token| {
      token
        .text()
        .is_some_and(|text| text.eq_ignore_ascii_case("reuse"))
    })
  }

  pub fn is_use(&self) -> bool {
    self.child_token(SyntaxKind::Ident).is_some_and(|token| {
      token
        .text()
        .is_some_and(|text| text.eq_ignore_ascii_case("use"))
    })
  }

  pub fn is_wildcard_use(&self) -> bool {
    self.child::<Wildcard>().is_some()
  }

  pub fn specifier_list(&self) -> Option<UseSpecifierList> {
    self.child::<UseSpecifierList>()
  }

  pub fn module_path(&self) -> Option<String> {
    if let Some(string) = self.child::<StringExpr>() {
      return Some(string.value());
    }
    self
      .0
      .children()
      .filter_map(|child| child.as_token())
      .find(|token| matches!(token.kind(), SyntaxKind::DqString | SyntaxKind::SqString))
      .and_then(|token| {
        token.text().map(|raw| {
          let string = raw.trim();
          if (string.starts_with('"') && string.ends_with('"'))
            || (string.starts_with('\'') && string.ends_with('\''))
          {
            interpret_string(&string[1..string.len() - 1])
          } else {
            string.to_string()
          }
        })
      })
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct Wildcard(RedNode);

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct UseSpecifierList(RedNode);

impl UseSpecifierList {
  pub fn specifiers(&self) -> impl Iterator<Item = UseSpecifier> {
    self.children::<UseSpecifier>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct UseSpecifier(RedNode);

impl UseSpecifier {
  /// The kind fragment (e.g. `schema` in `use { schema auth }`)
  pub fn kind(&self) -> Option<UseDeclarationTargetFragment> {
    self.children::<UseDeclarationTargetFragment>().next()
  }

  /// The name fragment (e.g. `auth` in `use { schema auth }`)
  pub fn name(&self) -> Option<UseDeclarationTargetFragment> {
    self.children::<UseDeclarationTargetFragment>().nth(1)
  }

  pub fn alias(&self) -> Option<UseSpecifierAlias> {
    self
      .children::<NameExpr>()
      .find_map(|expr| expr.child::<UseSpecifierAlias>())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct UseDeclarationTargetFragment(RedNode);

impl UseDeclarationTargetFragment {
  pub fn expr(&self) -> Option<Expr> {
    self.child::<Expr>()
  }

  pub fn path(&self) -> Vec<PathFragment> {
    self
      .expr()
      .and_then(|expr| expr.as_path())
      .unwrap_or_default()
  }

  pub fn name(&self) -> Option<PathFragment> {
    self.path().into_iter().last()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct UseSpecifierAlias(RedNode);

impl UseSpecifierAlias {
  pub fn segments(&self) -> impl Iterator<Item = NameExpr> {
    self.children::<NameExpr>()
  }
}

/* func / get / type Declarations */

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct FuncDeclaration(RedNode);

impl FuncDeclaration {
  pub fn name(&self) -> Option<FuncDeclarationName> {
    self.child::<FuncDeclarationName>()
  }

  pub fn params(&self) -> Option<FuncDeclarationParams> {
    self.child::<FuncDeclarationParams>()
  }

  pub fn return_typ(&self) -> Option<FuncDeclarationReturnTyp> {
    self.child::<FuncDeclarationReturnTyp>()
  }

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    self.child::<BlockElementDeclarationBody>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct FuncDeclarationName(RedNode);

impl FuncDeclarationName {
  pub fn is_operator_declaration(&self) -> bool {
    self.0.children().any(|child| {
      child.kind() == SyntaxKind::Ident && child.text().trim().eq_ignore_ascii_case("operator")
    })
  }

  pub fn operator_symbol(&self) -> Option<String> {
    if !self.is_operator_declaration() {
      return None;
    }
    self.0.children().find_map(|child| {
      if child.kind() == SyntaxKind::Operator {
        Some(child.text().trim().to_string())
      } else {
        None
      }
    })
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct FuncDeclarationParams(RedNode);

impl FuncDeclarationParams {
  pub fn params(&self) -> impl Iterator<Item = FuncDeclarationParam> {
    self.children::<FuncDeclarationParam>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct FuncDeclarationParam(RedNode);

impl FuncDeclarationParam {
  pub fn name(&self) -> Option<IdentExpr> {
    self.child::<IdentExpr>()
  }

  pub fn typ_expr(&self) -> Option<Expr> {
    self.children::<Expr>().nth(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct FuncDeclarationReturnTyp(RedNode);

impl FuncDeclarationReturnTyp {
  pub fn typ_expr(&self) -> Option<Expr> {
    self.child::<Expr>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct GetDeclaration(RedNode);

impl GetDeclaration {
  pub fn name(&self) -> Option<GetDeclarationName> {
    self.child::<GetDeclarationName>()
  }

  pub fn params(&self) -> Option<FuncDeclarationParams> {
    self.child::<FuncDeclarationParams>()
  }

  pub fn return_typ(&self) -> Option<FuncDeclarationReturnTyp> {
    self.child::<FuncDeclarationReturnTyp>()
  }

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    self.child::<BlockElementDeclarationBody>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct GetDeclarationName(RedNode);

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct EqualityDeclaration(RedNode);

impl EqualityDeclaration {
  pub fn name(&self) -> Option<ElementDeclarationTargetFragment> {
    self.child::<ElementDeclarationTargetFragment>()
  }

  pub fn setting_list(&self) -> Option<SettingList> {
    self.child::<SettingList>()
  }

  pub fn rhs(&self) -> Option<Expr> {
    self.child::<Expr>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct EqualityDeclarationName(RedNode);

/* Expressions */

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct InfixExpr(RedNode);

impl InfixExpr {
  pub fn left(&self) -> Option<Expr> {
    self.children::<Expr>().next()
  }

  pub fn right(&self) -> Option<Expr> {
    self.children::<Expr>().nth(1)
  }

  pub fn op_token(&self) -> Option<SyntaxToken> {
    self.child_token(SyntaxKind::Operator)
  }

  pub fn op(&self) -> Option<String> {
    self
      .op_token()
      .and_then(|token| token.text().map(ToString::to_string))
  }

  pub fn has_op(&self, op: &str) -> bool {
    self.op().as_deref() == Some(op)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct PrefixExpr(RedNode);

impl PrefixExpr {
  pub fn operand(&self) -> Option<Expr> {
    self.child::<Expr>()
  }

  pub fn op_token(&self) -> Option<SyntaxToken> {
    self.child_token(SyntaxKind::Operator)
  }

  pub fn op(&self) -> Option<String> {
    self
      .op_token()
      .and_then(|token| token.text().map(ToString::to_string))
  }

  pub fn has_op(&self, op: &str) -> bool {
    self.op().as_deref() == Some(op)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct PostfixExpr(RedNode);

impl PostfixExpr {
  pub fn operand(&self) -> Option<Expr> {
    self.child::<Expr>()
  }

  pub fn op_token(&self) -> Option<SyntaxToken> {
    self.child_token(SyntaxKind::Operator)
  }

  pub fn op(&self) -> Option<String> {
    self
      .op_token()
      .and_then(|token| token.text().map(ToString::to_string))
  }

  pub fn has_op(&self, op: &str) -> bool {
    self.op().as_deref() == Some(op)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct CallExpr(RedNode);

impl CallExpr {
  pub fn callee(&self) -> Option<Expr> {
    self.children::<Expr>().next()
  }

  pub fn args(&self) -> impl Iterator<Item = Expr> {
    self.children::<Expr>().skip(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct IndexExpr(RedNode);

impl IndexExpr {
  pub fn base(&self) -> Option<Expr> {
    self.children::<Expr>().next()
  }

  pub fn indices(&self) -> impl Iterator<Item = Expr> {
    self.children::<Expr>().skip(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ParenExpr(RedNode);

impl ParenExpr {
  pub fn inner(&self) -> Option<Expr> {
    self.child::<Expr>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ColorExpr(RedNode);

impl ColorExpr {
  /// Hex digit string from the token, without the `#` prefix
  fn digits(&self) -> String {
    self
      .child_token(SyntaxKind::ColorLit)
      .and_then(|token| {
        token
          .text()
          .map(|token| token.trim_start_matches('#').to_string())
      })
      .unwrap_or_default()
  }

  /// Parses a 1-digit shorthand channel (e.g. `"f"` -> `0xff`)
  fn parse_short_channel(string: &str) -> Option<u8> {
    u8::from_str_radix(&string.repeat(2), 16).ok()
  }

  /// Parses a 2-digit full channel (e.g. `"ff"` -> `0xff`)
  fn parse_full_channel(string: &str) -> Option<u8> {
    u8::from_str_radix(string, 16).ok()
  }

  /// Red component (0-255)
  pub fn r(&self) -> Option<u8> {
    let hex = self.digits();
    match hex.len() {
      3 => Self::parse_short_channel(&hex[0..1]),
      6 => Self::parse_full_channel(&hex[0..2]),
      _ => None,
    }
  }

  /// Green component (0-255)
  pub fn g(&self) -> Option<u8> {
    let hex = self.digits();
    match hex.len() {
      3 => Self::parse_short_channel(&hex[1..2]),
      6 => Self::parse_full_channel(&hex[2..4]),
      _ => None,
    }
  }

  /// Blue component (0-255)
  pub fn b(&self) -> Option<u8> {
    let hex = self.digits();
    match hex.len() {
      3 => Self::parse_short_channel(&hex[2..3]),
      6 => Self::parse_full_channel(&hex[4..6]),
      _ => None,
    }
  }

  /// Parses the literal as `(r, g, b)` u8 components from `#rgb` or `#rrggbb`
  pub fn value(&self) -> Option<(u8, u8, u8)> {
    let hex = self.digits();
    match hex.len() {
      3 => Some((
        Self::parse_short_channel(&hex[0..1])?,
        Self::parse_short_channel(&hex[1..2])?,
        Self::parse_short_channel(&hex[2..3])?,
      )),
      6 => Some((
        Self::parse_full_channel(&hex[0..2])?,
        Self::parse_full_channel(&hex[2..4])?,
        Self::parse_full_channel(&hex[4..6])?,
      )),
      _ => None,
    }
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ListExpr(RedNode);

impl ListExpr {
  pub fn items(&self) -> impl Iterator<Item = Expr> {
    self.children::<Expr>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct TupleExpr(RedNode);

impl TupleExpr {
  pub fn items(&self) -> impl Iterator<Item = Expr> {
    self.children::<Expr>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct CsvExpr(RedNode);

impl CsvExpr {
  pub fn items(&self) -> impl Iterator<Item = Expr> {
    self.children::<Expr>()
  }

  /// Returns one `Option<Expr>` per comma-separated slot
  /// Empty slots (consecutive commas like `1,,2`) yield `None`
  pub fn slots(&self) -> Vec<Option<Expr>> {
    let mut result = Vec::new();
    let mut has_expr_for_slot = false;
    for child in self.0.children() {
      if let Some(expr) = Expr::cast(child.clone()) {
        result.push(Some(expr));
        has_expr_for_slot = true;
      } else if child.kind() == SyntaxKind::Comma {
        if !has_expr_for_slot && !result.is_empty() {
          result.push(None);
        }
        has_expr_for_slot = false;
      }
    }
    result
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct NumberExpr(RedNode);

impl NumberExpr {
  /// Exact number token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    self.child_token(SyntaxKind::Number)
  }

  /// Raw text of the number token
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|token| token.text().map(ToString::to_string))
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

#[wrapper_ast_node(SyntaxKind = [IdentExpr, DqStringExpr])]
pub struct NameExpr(RedNode);

impl NameExpr {
  pub fn as_name(&self) -> Option<String> {
    if let Some(ident) = IdentExpr::cast(self.0.clone()) {
      Some(ident.name())
    } else {
      DqStringExpr::cast(self.0.clone()).map(|dq| dq.value())
    }
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct IdentExpr(RedNode);

impl IdentExpr {
  /// Exact identifier token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    self.child_token(SyntaxKind::Ident)
  }

  /// Identifier name
  pub fn name(&self) -> String {
    self
      .token()
      .and_then(|text| text.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct DqStringExpr(RedNode);

impl DqStringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    self.child_token(SyntaxKind::DqString)
  }

  /// Raw text of the string token including double quotes
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|token| token.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing double quotes
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    let string = raw.strip_prefix('"').unwrap_or(&raw);
    string.strip_suffix('"').unwrap_or(string).to_string()
  }

  /// Unescaped string value
  pub fn value(&self) -> String {
    interpret_string(&self.raw_text())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct SqStringExpr(RedNode);

impl SqStringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    self.child_token(SyntaxKind::SqString)
  }

  /// Raw text of the string token including single quotes
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|token| token.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing single quotes
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    let string = raw.strip_prefix('\'').unwrap_or(&raw);
    string.strip_suffix('\'').unwrap_or(string).to_string()
  }

  /// Unescaped string value
  pub fn value(&self) -> String {
    interpret_string(&self.raw_text())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct TqStringExpr(RedNode);

impl TqStringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    self.child_token(SyntaxKind::TqString)
  }

  /// Raw text of the string token including triple single quotes
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|token| token.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing triple single quotes
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    let string = raw.strip_prefix("'''").unwrap_or(&raw);
    string.strip_suffix("'''").unwrap_or(string).to_string()
  }

  /// Unescaped string value normalized by dedenting common indentation
  pub fn value(&self) -> String {
    interpret_string(&self.raw_text())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct OqStringExpr(RedNode);

impl OqStringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    self.child_token(SyntaxKind::OqString)
  }

  /// Raw text of the string token including backticks
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|token| token.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing backticks
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    let string = raw.strip_prefix('`').unwrap_or(&raw);
    string.strip_suffix('`').unwrap_or(string).to_string()
  }

  /// Unescaped string value where only backticks are escaped
  pub fn value(&self) -> String {
    interpret_string(&self.raw_text())
  }
}

#[wrapper_ast_node(SyntaxKind = [DqStringExpr, SqStringExpr, TqStringExpr, OqStringExpr])]
pub struct StringExpr(RedNode);

impl StringExpr {
  /// Exact string token excluding trivia
  pub fn token(&self) -> Option<SyntaxToken> {
    match self.0.kind() {
      SyntaxKind::DqStringExpr => self.child_token(SyntaxKind::DqString),
      SyntaxKind::SqStringExpr => self.child_token(SyntaxKind::SqString),
      SyntaxKind::TqStringExpr => self.child_token(SyntaxKind::TqString),
      SyntaxKind::OqStringExpr => self.child_token(SyntaxKind::OqString),
      _ => None,
    }
  }

  /// Raw text of the string token including quotes
  pub fn raw_text(&self) -> String {
    self
      .token()
      .and_then(|token| token.text().map(ToString::to_string))
      .unwrap_or_else(|| self.0.text().trim().to_string())
  }

  /// Inner content without enclosing quotes
  pub fn content(&self) -> String {
    let raw = self.raw_text();
    match self.0.kind() {
      SyntaxKind::TqStringExpr => {
        let string = raw.strip_prefix("'''").unwrap_or(&raw);
        string.strip_suffix("'''").unwrap_or(string).to_string()
      }
      SyntaxKind::DqStringExpr => {
        let string = raw.strip_prefix('"').unwrap_or(&raw);
        string.strip_suffix('"').unwrap_or(string).to_string()
      }
      SyntaxKind::SqStringExpr => {
        let string = raw.strip_prefix('\'').unwrap_or(&raw);
        string.strip_suffix('\'').unwrap_or(string).to_string()
      }
      SyntaxKind::OqStringExpr => {
        let string = raw.strip_prefix('`').unwrap_or(&raw);
        string.strip_suffix('`').unwrap_or(string).to_string()
      }
      _ => raw,
    }
  }

  /// Unescaped string value (normalized for multiline triple-quoted strings)
  pub fn value(&self) -> String {
    interpret_string(&self.raw_text())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ClosureExpr(RedNode);

impl ClosureExpr {
  /// Parameter expression: either TupleExpr or IdentExpr
  pub fn params(&self) -> Option<Expr> {
    self.children::<Expr>().next()
  }

  /// Body expression of the closure
  pub fn body(&self) -> Option<Expr> {
    self.children::<Expr>().nth(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct CommaExpr(RedNode);

impl CommaExpr {
  /// Elements in the comma-separated list
  pub fn items(&self) -> impl Iterator<Item = Expr> {
    self.children::<Expr>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ForallExpr(RedNode);

impl ForallExpr {
  pub fn binding(&self) -> Option<Expr> {
    self.children::<Expr>().next()
  }

  pub fn collection(&self) -> Option<Expr> {
    self.children::<Expr>().nth(1)
  }

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    self.child::<BlockElementDeclarationBody>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct ExistsExpr(RedNode);

impl ExistsExpr {
  pub fn binding(&self) -> Option<Expr> {
    self.children::<Expr>().next()
  }

  pub fn collection(&self) -> Option<Expr> {
    self.children::<Expr>().nth(1)
  }

  pub fn body(&self) -> Option<BlockElementDeclarationBody> {
    self.child::<BlockElementDeclarationBody>()
  }
}

/// Wrapper node unifying `ForallExpr` and `ExistsExpr` for shared quantifier logic
#[wrapper_ast_node(SyntaxKind = [ForallExpr, ExistsExpr])]
pub struct QuantifierExpr(RedNode);

impl QuantifierExpr {
  pub fn binding(&self) -> Option<String> {
    self.children::<Expr>().next()?.as_name()
  }

  pub fn collection(&self) -> Option<Expr> {
    self.children::<Expr>().nth(1)
  }

  pub fn body_expr(&self) -> Option<Expr> {
    self
      .child::<BlockElementDeclarationBody>()?
      .fields()
      .next()?
      .args()
      .next()?
      .expr()
  }

  pub fn is_forall(&self) -> bool {
    self.syntax().kind() == SyntaxKind::ForallExpr
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct TernaryExpr(RedNode);

impl TernaryExpr {
  pub fn condition(&self) -> Option<Expr> {
    self.children::<Expr>().next()
  }

  pub fn then_expr(&self) -> Option<Expr> {
    self.children::<Expr>().nth(1)
  }

  pub fn else_expr(&self) -> Option<Expr> {
    self.children::<Expr>().nth(2)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct MatchExpr(RedNode);

impl MatchExpr {
  pub fn subject(&self) -> Option<Expr> {
    self.children::<Expr>().next()
  }

  pub fn arms(&self) -> impl Iterator<Item = MatchArm> {
    self.children::<MatchArm>()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
pub struct MatchArm(RedNode);

impl MatchArm {
  pub fn pattern(&self) -> Option<Expr> {
    self.children::<Expr>().next()
  }

  pub fn result(&self) -> Option<Expr> {
    self.children::<Expr>().nth(1)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AstNode)]
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
  CsvExpr,
  NumberExpr,
  ColorExpr,
  IdentExpr,
  DqStringExpr,
  SqStringExpr,
  TqStringExpr,
  OqStringExpr,
  ClosureExpr,
  CommaExpr,
  ForallExpr,
  ExistsExpr,
  TernaryExpr,
  MatchExpr,
  BlockElementDeclaration,
  InlineElementDeclaration,
])]
pub struct Expr(RedNode);

impl Expr {
  pub fn unwrap_parens(&self) -> Expr {
    let mut current = self.clone();
    while let Some(paren) = ParenExpr::cast(current.0.clone()) {
      if let Some(inner) = paren.inner() {
        current = inner;
      } else {
        break;
      }
    }
    current
  }

  pub fn as_name(&self) -> Option<String> {
    let unwrapped = self.unwrap_parens();
    if let Some(ident) = IdentExpr::cast(unwrapped.0.clone()) {
      Some(ident.name())
    } else {
      DqStringExpr::cast(unwrapped.0).map(|dq| dq.value())
    }
  }

  pub fn as_path(&self) -> Option<Vec<PathFragment>> {
    let unwrapped = self.unwrap_parens();
    if let Some(name) = unwrapped.as_name() {
      return Some(vec![PathFragment::new(unwrapped.syntax().clone(), name)]);
    }
    let infix = InfixExpr::cast(unwrapped.0)?;
    if infix.has_op(".") {
      let mut left_path = infix.left()?.as_path()?;
      let right_path = infix.right()?.as_path()?;
      left_path.extend(right_path);
      return Some(left_path);
    }
    None
  }

  pub fn is_path_expr(&self) -> bool {
    let unwrapped = self.unwrap_parens();

    // Name expr is a path expr
    if unwrapped.as_name().is_some() {
      return true;
    }

    // path_expr = path_expr `.` name_expr
    let Some(infix) = InfixExpr::cast(unwrapped.0) else {
      return false;
    };
    infix.has_op(".") && infix.left().is_some_and(|left| left.is_path_expr())
  }

  pub fn as_string(&self) -> Option<String> {
    let unwrapped = self.unwrap_parens();
    StringExpr::cast(unwrapped.0).map(|string| string.value())
  }

  pub fn as_number(&self) -> Option<f64> {
    let unwrapped = self.unwrap_parens();
    NumberExpr::cast(unwrapped.0).and_then(|number| number.value())
  }

  pub fn as_int(&self) -> Option<i64> {
    let unwrapped = self.unwrap_parens();
    NumberExpr::cast(unwrapped.0).and_then(|number| number.int_value())
  }

  pub fn as_bool(&self) -> Option<bool> {
    let ident = self.as_name()?;
    match ident.as_str() {
      "true" => Some(true),
      "false" => Some(false),
      _ => None,
    }
  }
}

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
    let result = ctx.parse();
    let root = RedNode::new_root(result.ast);
    let source_file = SourceFile::cast(root).expect("SourceFile cast");

    let table: BlockElementDeclaration = source_file
      .element_declarations()
      .next()
      .expect("Table element")
      .try_into()
      .expect("BlockElementDeclaration cast");
    let table_element = ElementDeclaration::cast(table.0.clone()).expect("ElementDeclaration cast");
    assert_eq!(table_element.typ().unwrap().text().trim(), "Table");
    assert_eq!(table_element.target_name().unwrap().text().trim(), "users");
    assert_eq!(table_element.alias().unwrap().text().trim(), "u");

    let settings = table_element.setting_list().expect("setting list");
    let note_item = settings.get_item("note").expect("find note item");
    assert_eq!(note_item.name().unwrap().text().trim(), "note");
    assert_eq!(note_item.value().unwrap().text().trim(), "'user table'");

    let body = table.body().expect("table body");
    let field = body.fields().next().expect("id field");
    let field_exprs: Vec<_> = field.args().collect();
    assert_eq!(field_exprs.len(), 2);
    assert_eq!(field_exprs[0].text().trim(), "id");
    assert_eq!(field_exprs[1].text().trim(), "integer");

    let attr = body.attributes().next().expect("note attribute");
    assert_eq!(attr.name().unwrap().text().trim(), "note");
    assert_eq!(attr.value().unwrap().text().trim(), "'user note'");

    let nested = body.element_declarations().next().expect("nested indexes");
    let nested_element =
      ElementDeclaration::cast(nested.0.clone()).expect("ElementDeclaration cast");
    assert_eq!(nested_element.typ().unwrap().text().trim(), "indexes");
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

    let tq = parse_expr_node(
      r#"'''
    line 1
    line 2
'''"#,
    );
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

    let number = parse_expr_node("42");
    let number_expr = NumberExpr::cast(number).expect("NumberExpr");
    assert_eq!(number_expr.value(), Some(42.0));
    assert_eq!(number_expr.int_value(), Some(42));

    let float_number = parse_expr_node("12.5");
    let float_expr = NumberExpr::cast(float_number).expect("NumberExpr float");
    assert_eq!(float_expr.value(), Some(12.5));
    assert_eq!(float_expr.int_value(), None);

    // Verify robustness against trivia inside or surrounding expressions
    let number_with_trivia = parse_expr_node("/* comment */ 100");
    let number_trivia_expr = NumberExpr::cast(number_with_trivia).expect("NumberExpr with trivia");
    assert_eq!(number_trivia_expr.raw_text(), "100");
    assert_eq!(number_trivia_expr.value(), Some(100.0));
    assert_eq!(number_trivia_expr.int_value(), Some(100));

    let string_with_trivia = parse_expr_node(r#"/* note */ "escaped\ncontent""#);
    let string_trivia_expr =
      DqStringExpr::cast(string_with_trivia).expect("DqStringExpr with trivia");
    assert_eq!(string_trivia_expr.raw_text(), "\"escaped\\ncontent\"");
    assert_eq!(string_trivia_expr.value(), "escaped\ncontent");
  }

  #[test]
  fn test_declaration_kinds() {
    let input = r#"
use { table users as u } from './users.dbml'

Table users as u [note: 'user table', pk] {
  id integer [increment]
  note: 'user note'
  indexes {
    id
  }
  fn calculate(x: int): int {
    x
  }
}

type UserId = int
"#;
    let cache = Rc::new(RefCell::new(Cache::new()));
    let stream: PeekableStream<'_, char> =
      itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
    let ctx = ParseCtx::new(stream, cache);
    let result = ctx.parse();
    let root = RedNode::new_root(result.ast);
    let source_file = SourceFile::cast(root).expect("SourceFile cast");

    let all_declarations: Vec<_> = source_file.declarations().collect();
    assert_eq!(all_declarations.len(), 3);
    assert!(UseDeclaration::try_from(all_declarations[0].clone()).is_ok());
    assert!(BlockElementDeclaration::try_from(all_declarations[1].clone()).is_ok());
    assert!(EqualityDeclaration::try_from(all_declarations[2].clone()).is_ok());

    let use_declaration_ref: Result<UseDeclaration, _> = (&all_declarations[0]).try_into();
    assert!(use_declaration_ref.is_ok());
    let element_declaration_ref: Result<BlockElementDeclaration, _> =
      (&all_declarations[1]).try_into();
    assert!(element_declaration_ref.is_ok());
  }

  #[test]
  fn test_use_declaration() {
    let input = r#"
use { table users as u } from './users.dbml'

Table users as u [note: 'user table', pk] {
  id integer [increment]
  note: 'user note'
  indexes {
    id
  }
  fn calculate(x: int): int {
    x
  }
}

type UserId = int
"#;
    let cache = Rc::new(RefCell::new(Cache::new()));
    let stream: PeekableStream<'_, char> =
      itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
    let ctx = ParseCtx::new(stream, cache);
    let res = ctx.parse();
    let root = RedNode::new_root(res.ast);
    let source_file = SourceFile::cast(root).expect("SourceFile cast");

    let use_declaration = source_file.use_declarations().next().expect("use decl");
    assert_eq!(
      use_declaration.module_path().as_deref(),
      Some("./users.dbml")
    );
    assert!(!use_declaration.is_reuse());
    let specifier = use_declaration
      .specifier_list()
      .expect("specifier list")
      .specifiers()
      .next()
      .expect("specifier");
    assert_eq!(
      specifier
        .kind()
        .map(|kind| kind.text().trim().to_string())
        .as_deref(),
      Some("table")
    );
    assert_eq!(
      specifier
        .name()
        .map(|name| name.text().trim().to_string())
        .as_deref(),
      Some("users")
    );
    assert_eq!(
      specifier
        .alias()
        .map(|alias| alias.text().trim().to_string())
        .as_deref(),
      Some("u")
    );
  }

  #[test]
  fn test_element_declaration() {
    let input = r#"
use { table users as u } from './users.dbml'

Table users as u [note: 'user table', pk] {
  id integer [increment]
  note: 'user note'
  indexes {
    id
  }
  fn calculate(x: int): int {
    x
  }
}

type UserId = int
"#;
    let cache = Rc::new(RefCell::new(Cache::new()));
    let stream: PeekableStream<'_, char> =
      itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
    let ctx = ParseCtx::new(stream, cache);
    let result = ctx.parse();
    let root = RedNode::new_root(result.ast);
    let source_file = SourceFile::cast(root).expect("SourceFile cast");

    let element = source_file.element_declarations().next().expect("element");
    assert_eq!(element.typ().unwrap().text().trim(), "Table");
    assert_eq!(element.target_name().unwrap().text().trim(), "users");
    assert_eq!(element.alias().unwrap().text().trim(), "u");

    let settings = element.setting_list().expect("settings");
    assert!(settings.has_item("pk"));
    assert_eq!(
      settings
        .get_item("note")
        .and_then(|item| item.value())
        .and_then(|value| value.expr())
        .and_then(|expr| expr.as_string())
        .as_deref(),
      Some("user table")
    );
    assert_eq!(
      settings.get_item("pk").map(|item| item.value().is_none()),
      Some(true)
    );
  }

  #[test]
  fn test_block_body() {
    let input = r#"
use { table users as u } from './users.dbml'

Table users as u [note: 'user table', pk] {
  id integer [increment]
  note: 'user note'
  indexes {
    id
  }
  fn calculate(x: int): int {
    x
  }
}

type UserId = int
"#;
    let cache = Rc::new(RefCell::new(Cache::new()));
    let stream: PeekableStream<'_, char> =
      itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
    let ctx = ParseCtx::new(stream, cache);
    let result = ctx.parse();
    let root = RedNode::new_root(result.ast);
    let source_file = SourceFile::cast(root).expect("SourceFile cast");

    let element = source_file.element_declarations().next().expect("element");
    let block: BlockElementDeclaration = element.try_into().expect("block element");
    let body = block.body().expect("body");
    let items: Vec<_> = body.items().collect();
    assert_eq!(items.len(), 4);
    assert!(ElementFieldDeclaration::try_from(items[0].clone()).is_ok());
    assert!(ElementAttributeDeclaration::try_from(items[1].clone()).is_ok());
    assert!(BlockElementDeclaration::try_from(items[2].clone()).is_ok());
    assert!(FuncDeclaration::try_from(items[3].clone()).is_ok());

    let field = body.fields().next().expect("field");
    assert_eq!(
      field
        .args()
        .next()
        .map(|expr| expr.text().trim().to_string())
        .as_deref(),
      Some("id")
    );
    assert_eq!(
      field
        .args()
        .nth(1)
        .map(|expr| expr.text().trim().to_string())
        .as_deref(),
      Some("integer")
    );
    assert!(field.setting_list().unwrap().has_item("increment"));

    let attribute = body.attributes().next().expect("attribute");
    assert_eq!(attribute.name().unwrap().text().trim(), "note");
    assert_eq!(
      attribute
        .value()
        .and_then(|value| value.expr())
        .and_then(|expr| expr.as_string())
        .as_deref(),
      Some("user note")
    );

    let func_declaration = body.func_declarations().next().expect("fn decl");
    assert_eq!(func_declaration.name().unwrap().text().trim(), "calculate");
    let param = func_declaration
      .params()
      .unwrap()
      .params()
      .next()
      .expect("param");
    assert_eq!(
      param
        .name()
        .map(|expr| expr.text().trim().to_string())
        .as_deref(),
      Some("x")
    );
    assert_eq!(
      param
        .typ_expr()
        .map(|expr| expr.text().trim().to_string())
        .as_deref(),
      Some("int")
    );
    assert_eq!(
      func_declaration
        .return_typ()
        .and_then(|return_typ| return_typ.typ_expr())
        .unwrap()
        .text()
        .trim(),
      "int"
    );

    let typ_declaration = source_file
      .equality_declarations()
      .next()
      .expect("type decl");
    assert_eq!(typ_declaration.name().unwrap().text().trim(), "UserId");
  }

  #[test]
  fn test_expr_api() {
    let path_node = parse_expr_node("schema.users.id");
    let path_expr = Expr::cast(path_node).expect("Expr cast");
    assert_eq!(
      path_expr.as_path().map(|fragments| fragments
        .into_iter()
        .map(|fragment| fragment.name().to_string())
        .collect::<Vec<_>>()),
      Some(vec![
        "schema".to_string(),
        "users".to_string(),
        "id".to_string()
      ])
    );

    let single_node = parse_expr_node("users");
    let single_expr = Expr::cast(single_node).expect("Expr cast");
    assert_eq!(
      single_expr.as_path().map(|fragments| fragments
        .into_iter()
        .map(|fragment| fragment.name().to_string())
        .collect::<Vec<_>>()),
      Some(vec!["users".to_string()])
    );
    assert_eq!(single_expr.as_name().as_deref(), Some("users"));

    let paren_node = parse_expr_node("((42))");
    let paren_expr = Expr::cast(paren_node).expect("Expr cast");
    assert_eq!(paren_expr.as_number(), Some(42.0));
    assert_eq!(paren_expr.as_int(), Some(42));

    let bool_node = parse_expr_node("true");
    let bool_expr = Expr::cast(bool_node).expect("Expr cast");
    assert_eq!(bool_expr.as_bool(), Some(true));

    let string_node = parse_expr_node(r#""hello\nworld""#);
    let string_expr = Expr::cast(string_node).expect("Expr cast");
    assert_eq!(string_expr.as_string().as_deref(), Some("hello\nworld"));

    let infix_node = parse_expr_node("1 + 2");
    let infix_expr: InfixExpr = Expr::cast(infix_node)
      .expect("Expr cast")
      .try_into()
      .expect("Infix cast");
    assert_eq!(infix_expr.op().as_deref(), Some("+"));
    assert_eq!(infix_expr.left().unwrap().as_number(), Some(1.0));
    assert_eq!(infix_expr.right().unwrap().as_number(), Some(2.0));
  }

  fn parse_expr_node(input: &str) -> RedNode {
    let cache = Rc::new(RefCell::new(Cache::new()));
    let stream: PeekableStream<'_, char> =
      itertools::multipeek(Box::new(input.chars()) as Box<dyn Iterator<Item = char> + '_>);
    let mut ctx = ParseCtx::new(stream, cache);
    let (green, _) = ctx.expr();
    RedNode::new_root(green)
  }
}
