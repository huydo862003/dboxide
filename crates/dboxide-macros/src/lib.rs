use proc_macro::TokenStream;

mod ast;
mod diagnostic;
mod enum_variant;

#[proc_macro_derive(AstNode, attributes(ast_node))]
pub fn ast_node_derive(item: TokenStream) -> TokenStream {
  ast::ast_node_derive_impl(item)
}

/// Attribute macro for synthetic AST nodes that wrap multiple SyntaxKind variants
/// Usage:
/// ```ignore
/// #[wrapper_ast_node(SyntaxKind = [BlockElementDeclaration, InlineElementDeclaration])]
/// pub struct ElementDeclaration(RedNode);
/// ```
#[proc_macro_attribute]
pub fn wrapper_ast_node(attribute: TokenStream, item: TokenStream) -> TokenStream {
  ast::wrapper_ast_node_impl(attribute, item)
}

#[proc_macro_attribute]
pub fn dboxide_diagnostic_kind(attribute: TokenStream, item: TokenStream) -> TokenStream {
  diagnostic::dboxide_diagnostic_kind_impl(attribute, item)
}

/// Attribute macro that generates a companion `{Name}Kind` enum (unit variants, no payloads)
/// and a `variant_kind(&self) -> {Name}Kind` method on the annotated enum
#[proc_macro_attribute]
pub fn enum_variant(attribute: TokenStream, item: TokenStream) -> TokenStream {
  enum_variant::enum_variant_impl(attribute, item)
}
