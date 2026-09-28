//! Macros for the AST layer in dboxide-syntax

use proc_macro::TokenStream;
use quote::quote;
use syn::{
  Ident, Token,
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
};

pub fn ast_node_derive_impl(item: TokenStream) -> TokenStream {
  let item_ast: syn::DeriveInput = syn::parse(item).unwrap();

  let name = &item_ast.ident;

  // Check for optional #[ast_node(KIND)] attribute
  let mut kind_ident = None;
  for attr in &item_ast.attrs {
    if attr.path().is_ident("ast_node") {
      let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("SyntaxKind") {
          let value = meta.value()?;
          let ident: Ident = value.parse()?;
          kind_ident = Some(ident);
        } else if let Some(ident) = meta.path.get_ident() {
          kind_ident = Some(ident.clone());
        }
        Ok(())
      });
    }
  }

  let kind = kind_ident.unwrap_or_else(|| name.clone());

  let generated = quote! {
    impl crate::syntax::ast::AstNode for #name {
      fn cast(syntax: crate::syntax::ast::RedNode) -> Option<Self> {
        match syntax.kind() {
          crate::syntax::ast::SyntaxKind::#kind => Some(Self(syntax)),
          _ => None,
        }
      }
      fn syntax(&self) -> &crate::syntax::ast::RedNode {
        &self.0
      }
    }
  };
  generated.into()
}

// Parses: SyntaxKind = [Variant1, Variant2, ...]
struct WrapperAstNodeMacroArgs {
  kinds: Punctuated<Ident, Token![,]>,
}

impl Parse for WrapperAstNodeMacroArgs {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let _: Ident = input.parse()?; // Consume SyntaxKind
    let _: Token![=] = input.parse()?; // Consume `=`

    let content;
    syn::bracketed!(content in input); // Extract the list content

    // Parse the list content and return
    let kinds = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
    Ok(WrapperAstNodeMacroArgs { kinds })
  }
}

pub fn wrapper_ast_node_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
  let args: WrapperAstNodeMacroArgs = syn::parse(attr).unwrap();
  let item_ast: syn::DeriveInput = syn::parse(item).unwrap();

  let name = &item_ast.ident;
  let kinds: Vec<_> = args.kinds.iter().collect();

  let from_impls = kinds.iter().map(|kind| {
    quote! {
      impl From<#kind> for #name {
        fn from(node: #kind) -> Self {
          Self(crate::syntax::ast::AstNode::syntax(&node).clone())
        }
      }

      impl TryFrom<#name> for #kind {
        type Error = ();
        fn try_from(node: #name) -> Result<Self, ()> {
          crate::syntax::ast::AstNode::cast(node.0).ok_or(())
        }
      }
    }
  });

  let generated = quote! {
    #[derive(Clone, PartialEq, Eq, Hash)]
    #item_ast

    impl crate::syntax::ast::AstNode for #name {
      fn cast(syntax: crate::syntax::ast::RedNode) -> Option<Self> {
        match syntax.kind() {
          #(crate::syntax::ast::SyntaxKind::#kinds)|* => Some(Self(syntax)),
          _ => None,
        }
      }
      fn syntax(&self) -> &crate::syntax::ast::RedNode {
        &self.0
      }
    }

    #(#from_impls)*
  };
  generated.into()
}
