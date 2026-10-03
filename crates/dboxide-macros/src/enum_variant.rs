use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Attribute macro that generates a companion `{Name}Kind` enum (unit variants, no payloads)
/// and a `variant_kind(&self) -> {Name}Kind` method on the annotated enum
///
/// Example:
/// ```ignore
/// #[enum_variant]
/// pub enum Foo<'a> {
///   Bar,
///   Baz(u32),
/// }
/// // generates:
/// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// pub enum FooKind { Bar, Baz }
/// impl<'a> Foo<'a> {
///   pub fn variant_kind(&self) -> FooKind {
///     match self { Self::Bar => FooKind::Bar, Self::Baz(..) => FooKind::Baz }
///   }
/// }
/// ```
pub fn enum_variant_impl(_attribute: TokenStream, item: TokenStream) -> TokenStream {
  let original = TokenStream2::from(item.clone());
  let input = parse_macro_input!(item as DeriveInput);
  let name = &input.ident;
  let kind_name = syn::Ident::new(&format!("{}Kind", name), name.span());
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  let Data::Enum(data) = &input.data else {
    return syn::Error::new_spanned(name, "enum_variant can only be used on enums")
      .to_compile_error()
      .into();
  };

  let mut kind_variants = Vec::new();
  let mut match_arms = Vec::new();

  for variant in &data.variants {
    let variant_name = &variant.ident;
    kind_variants.push(quote! { #variant_name, });

    let pattern = match &variant.fields {
      Fields::Unit => quote! { Self::#variant_name },
      Fields::Unnamed(_) => quote! { Self::#variant_name(..) },
      Fields::Named(_) => quote! { Self::#variant_name { .. } },
    };

    match_arms.push(quote! { #pattern => #kind_name::#variant_name });
  }

  quote! {
    #original

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum #kind_name {
      #(#kind_variants)*
    }

    impl #impl_generics #name #ty_generics #where_clause {
      pub fn variant_kind(&self) -> #kind_name {
        match self {
          #(#match_arms,)*
        }
      }
    }
  }
  .into()
}
