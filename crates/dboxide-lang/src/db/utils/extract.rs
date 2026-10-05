use crate::ast::{
  AstNode, BlockElementDeclaration, BlockElementDeclarationBody, ElementAttributeDeclaration,
  ElementDeclaration, ElementFieldDeclaration, EqualityDeclaration, FnDeclaration, GetDeclaration,
};

/// Extracted from a typ definition body (`type Table [element] { ... }`)
#[derive(Default)]
pub struct TypBodyExtract {
  /// Field declarations
  pub field_declarations: Vec<ElementFieldDeclaration>,
  /// Attribute declarations (`key: value`)
  pub attributes: Vec<ElementAttributeDeclaration>,
  /// Namespace block body (if present)
  pub namespace_body: Option<BlockElementDeclarationBody>,
  /// Constraints block (`constraints { ... }`)
  pub constraints: Option<BlockElementDeclaration>,
  /// Nested type definitions (`type Column [field] { ... }`)
  pub typ_declarations: Vec<BlockElementDeclaration>,
  /// Nested type aliases (`type Name = expr`)
  pub typ_aliases: Vec<EqualityDeclaration>,
  /// Function declarations
  pub function_declarations: Vec<FnDeclaration>,
  /// Getter declarations (`get name(): ReturnType { body }`)
  pub getter_declarations: Vec<GetDeclaration>,
}

/// Extracted from an element instance body (`Table users { ... }`)
#[derive(Default)]
pub struct ElementBodyExtract {
  /// Field declarations (positional args per line)
  pub field_declarations: Vec<ElementFieldDeclaration>,
  /// Key-value attributes (`note: 'string'`, `headercolor: #ff0000`)
  pub attributes: Vec<ElementAttributeDeclaration>,
  /// Nested sub-elements (`indexes { ... }`, `checks { ... }`)
  pub element_declarations: Vec<BlockElementDeclaration>,
}

/// Extract items from a typ definition body
pub fn extract_typ_body(body: &BlockElementDeclarationBody) -> TypBodyExtract {
  let field_declarations: Vec<_> = body.fields().collect();
  let attributes = body.attributes().collect();
  let mut namespace_body = None;
  let mut constraints = None;
  let mut typ_declarations = Vec::new();
  let typ_aliases = body.equality_declarations().collect();
  let function_declarations = body.fn_declarations().collect();
  let getter_declarations = body.get_declarations().collect();

  for nested in body.element_declarations() {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    let typ_name = element.typ().map(|typ| typ.text().trim().to_lowercase());

    match typ_name.as_deref() {
      Some("namespace") => {
        if let Some(nested_block) = BlockElementDeclaration::cast(element.syntax().clone()) {
          namespace_body = nested_block.body();
        }
      }
      Some("constraints") => constraints = Some(nested),
      Some("type") => typ_declarations.push(nested),
      _ => {}
    }
  }

  TypBodyExtract {
    field_declarations,
    attributes,
    namespace_body,
    constraints,
    typ_declarations,
    typ_aliases,
    function_declarations,
    getter_declarations,
  }
}

/// Extract items from an element instance body
pub fn extract_element_body(body: &BlockElementDeclarationBody) -> ElementBodyExtract {
  let field_declarations = body.fields().collect();
  let attributes = body.attributes().collect();
  let element_declarations = body.element_declarations().collect();

  ElementBodyExtract {
    field_declarations,
    attributes,
    element_declarations,
  }
}
