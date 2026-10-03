use std::collections::HashMap;

use itertools::Either;
use salsa::{Accumulator, Database, tracked};

use crate::ast::{
  AstNode, BlockElementDeclaration, ElementDeclaration, Expr, InfixExpr, InlineElementDeclaration,
  RedNode, SourceFile, UseDeclaration, UseDeclarationTargetFragment,
};
use crate::db::types::Diagnostics;
use crate::diagnostics::DiagnosticKind;
use crate::parse_file::parse_file;
use crate::utils::{check_import_collision, does_symbol_match_kind, is_keyword_kind};
use crate::{get_declaration_typ, is_element_symbol, is_operator_symbol};

use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind};
use crate::db::types::{File, Symbol};

use super::declare::{declare_element_symbols, declare_func_symbol};
use crate::db::pretype_analyze::{pretype_get_file_symbols, pretype_resolve_referee};
use crate::db::pretype_evaluate_typ::pretype_evaluate_typ_symbol;
use crate::db::types::{Obj, OnUsePolicy, Typ};

/// Collects all symbols visible in a file at posttype scope (extends pretype with elements, fns, operators)
#[tracked]
pub fn posttype_get_file_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  reuses_only: bool,
) -> StaticScopeMembers<'db> {
  let mut members = pretype_get_file_symbols(db, file, reuses_only).clone(); // Pretype members

  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return members;
  };

  let mut seen_unique_symbols: HashMap<(Symbol, String), RedNode> = HashMap::new(); // Map from (type, name) to the first found such symbol

  /* Scan owned element declarations */
  for element_declaration in source_file.element_declarations() {
    // Already collected in pretype
    if element_declaration.is_typ_declaration() {
      continue;
    }

    // Check if this element declare a symbol
    let declared = declare_element_symbols(db, file, element_declaration.syntax());
    for symbol in &declared {
      members.insert(db, *symbol);
    }

    // Some elements can be unique, some are anonymous/can be duplicated
    if let Some((typ_symbol, label)) = resolve_unique_label(db, file, &element_declaration) {
      let symbol_key = (typ_symbol, label.clone());

      let node = element_declaration.syntax();

      // Check if this symbol is already seen
      if let Some(first_node) = seen_unique_symbols.get(&symbol_key) {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::DuplicateElementLabel {
              name: label.clone(),
              typ_name: element_declaration
                .typ()
                .map(|typ| typ.text())
                .unwrap_or_default(),
              first_offset: first_node.start_offset(),
            }
            .into_error(node.start_offset(), node.end_offset()),
          ),
          db,
        );
      } else {
        seen_unique_symbols.insert(symbol_key, node.clone());
      }
    }
  }

  /* Collect nested anonymous elements from block bodies for types with FollowRefs policy */
  for element_declaration in source_file.element_declarations() {
    if element_declaration.is_typ_declaration() {
      continue;
    }
    if let Some(block) = BlockElementDeclaration::cast(element_declaration.syntax().clone()) {
      collect_nested_anonymous_elements(db, file, &block, &mut members);
    }
  }

  /* SKIP - Scan owned equality declarations are skipped, as they can only be types for now */

  /* Scan use declarations */
  for use_declaration in source_file.use_declarations() {
    // If not resolving public symbols, skip non reuses
    if reuses_only && !use_declaration.is_reuse() {
      continue;
    }

    // Pull elements
    import_element_instances(db, file, &use_declaration, &mut members);
  }

  /* Scan function declarations */
  for func_declaration in source_file.func_declarations() {
    if let Some(symbol) = declare_func_symbol(db, file, &func_declaration) {
      members.insert(db, symbol);
    }
  }

  members
}

/// Imports element instances and operators from a use declaration into `members`
fn import_element_instances<'db>(
  db: &'db dyn Database,
  file: File,
  use_declaration: &UseDeclaration,
  members: &mut StaticScopeMembers<'db>,
) {
  /* Extract the used module */
  let Some(specifier_path) = use_declaration.module_path() else {
    return;
  };

  let Some(module) = file.uses(db).get(&specifier_path) else {
    return;
  };

  // Either::Left(virtual_module) currently can only be metatype, which doesn't have element instances
  let Either::Right(target_file) = module else {
    return;
  };

  let target_symbols = posttype_get_file_symbols(db, *target_file, true).clone();

  /* Handle use * */
  if use_declaration.is_wildcard_use() {
    let mut deferred_anonymous: Vec<Symbol> = Vec::new();

    /* First pass: import named elements and operators */
    for (_, original_symbol) in target_symbols.iter() {
      if !is_element_symbol(db, *original_symbol) && !is_operator_symbol(db, *original_symbol) {
        continue;
      }
      if matches!(original_symbol.kind(db), SymbolKind::AnonymousElement(..)) {
        deferred_anonymous.push(*original_symbol);
        continue;
      }
      import_wildcard_symbol(db, file, use_declaration, members, *original_symbol);
    }

    /* Second pass: import anonymous elements only if all their referenced names are now in scope */
    for original_symbol in deferred_anonymous {
      let SymbolKind::AnonymousElement(_, element_node) = original_symbol.kind(db) else {
        continue;
      };
      let references = extract_root_names_from_element(element_node);
      if references
        .iter()
        .all(|name| members.lookup_by_name(name).next().is_some())
      {
        import_wildcard_symbol(db, file, use_declaration, members, original_symbol);
      }
    }

    return;
  }

  /* Handle use {} */
  let Some(specifier_list) = use_declaration.specifier_list() else {
    return;
  };

  for specifier in specifier_list.specifiers() {
    let kind_fragment: Option<UseDeclarationTargetFragment> = specifier.kind(); // Kind of the imported symbol

    let Some(specifier_name) = specifier.name().and_then(|node| node.name()) else {
      continue;
    };
    let specifier_name = specifier_name.name().to_string(); // Name of the imported symbol

    let alias = specifier.alias();
    let visible_name = alias
      .as_ref()
      .map(|alias_node| alias_node.text().trim().to_string())
      .unwrap_or_else(|| specifier_name.clone()); // Visible name in this file

    /* Validate user-defined kind specifier */
    let kind_is_keyword = kind_fragment.as_ref().is_some_and(is_keyword_kind);
    if let Some(ref kind) = kind_fragment
      && !kind_is_keyword
    {
      let kind_name = kind
        .name()
        .map(|fragment| fragment.name().to_string())
        .unwrap_or_default();
      let resolved = kind
        .name()
        .and_then(|name_node| *pretype_resolve_referee(db, file, name_node.node()));
      if let Some(kind_symbol) = resolved
        && let Some(Obj::Typ(Typ::Element(element_typ))) =
          *pretype_evaluate_typ_symbol(db, kind_symbol)
        && !element_typ
          .label_definitions(db)
          .iter()
          .any(|label| label.unique && !label.is_alias)
      {
        let node = kind.syntax();
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::ImportKindMustBeUnique { kind_name }
              .into_error(node.start_offset(), node.end_offset()),
          ),
          db,
        );
        continue;
      }
    }

    // All matched symbols of the name + kind
    let matching_symbols: Vec<Symbol> = target_symbols
      .lookup_by_name(&specifier_name)
      .filter(|(_, symbol)| {
        is_element_symbol(db, **symbol)
          && kind_fragment
            .as_ref()
            .is_none_or(|kind| does_symbol_match_kind(db, file, kind, **symbol))
      })
      .map(|(_, symbol)| *symbol)
      .collect();

    for original_symbol in matching_symbols {
      let node = specifier.syntax();
      if !check_import_collision(
        db,
        members,
        &visible_name,
        original_symbol,
        node.start_offset(),
        node.end_offset(),
      ) {
        continue;
      }
      let use_symbol = Symbol::new(
        db,
        SymbolKind::Use(
          original_symbol,
          file,
          specifier.syntax().clone(),
          visible_name.clone(),
        ),
      );
      members.insert(db, use_symbol);
      expand_on_use_if_needed(db, original_symbol, file, specifier.syntax(), members);
    }
  }
}

/// Imports a single symbol under its original name into `members`, checking for collisions
fn import_wildcard_symbol<'db>(
  db: &'db dyn Database,
  file: File,
  use_declaration: &UseDeclaration,
  members: &mut StaticScopeMembers<'db>,
  original_symbol: Symbol<'db>,
) {
  let visible_name = original_symbol.name(db).to_string();
  let node = use_declaration.syntax();
  if !check_import_collision(
    db,
    members,
    &visible_name,
    original_symbol,
    node.start_offset(),
    node.end_offset(),
  ) {
    return;
  }
  let use_symbol = Symbol::new(
    db,
    SymbolKind::Use(
      original_symbol,
      file,
      use_declaration.syntax().clone(),
      visible_name,
    ),
  );
  members.insert(db, use_symbol);
  expand_on_use_if_needed(db, original_symbol, file, use_declaration.syntax(), members);
}

/// Extracts the root (leftmost) identifier names from all expressions in an anonymous element's inline field
fn extract_root_names_from_element(element_node: &RedNode) -> Vec<String> {
  let Some(inline) = InlineElementDeclaration::cast(element_node.clone()) else {
    return vec![];
  };
  let Some(field) = inline.field() else {
    return vec![];
  };
  let mut names = Vec::new();
  for arg in field.args() {
    if let Some(expr) = arg.expr() {
      collect_root_names(&expr, &mut names);
    }
  }
  names
}

/// Collects the leftmost identifier in each leaf of a dot-expression tree
fn collect_root_names(expr: &Expr, out: &mut Vec<String>) {
  if let Some(infix) = InfixExpr::cast(expr.syntax().clone()) {
    if infix.has_op(".") {
      // Only descend into LHS for dot: `a.b` -> root is `a`
      if let Some(left) = infix.left() {
        collect_root_names(&left, out);
      }
      return;
    }
    // Other infix (e.g. `>`, `<`): collect roots from both sides
    if let Some(left) = infix.left() {
      collect_root_names(&left, out);
    }
    if let Some(right) = infix.right() {
      collect_root_names(&right, out);
    }
    return;
  }
  if let Some(name) = expr.as_name() {
    out.push(name);
  }
}

/// When an imported element has `[use: expand namespaces]`, resolve references in the element's body and inject them into the importing scope
fn expand_on_use_if_needed<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
  importing_file: File,
  use_node: &RedNode,
  members: &mut StaticScopeMembers<'db>,
) {
  let resolved = symbol.original_symbol(db);
  let SymbolKind::UserElement(element_file, element_node, _) = resolved.kind(db) else {
    return;
  };
  let element_file = *element_file;
  let Some(Typ::Element(element_typ)) =
    *get_declaration_typ(db, element_file, element_node.clone())
  else {
    return;
  };
  if element_typ.typ_settings(db).on_use != OnUsePolicy::ExpandReferences {
    return;
  }

  /* Resolve field line arg names against the source file's posttype scope */
  let source_scope = posttype_get_file_symbols(db, element_file, false);
  let Some(block) = BlockElementDeclaration::cast(element_node.clone()) else {
    return;
  };
  let Some(body) = block.body() else { return };

  for field_line in body.fields() {
    for arg in field_line.args() {
      let Some(name) = arg.expr().and_then(|element| element.as_name()) else {
        continue;
      };
      for (_, ref_symbol) in source_scope.lookup_by_name(&name) {
        if !is_element_symbol(db, *ref_symbol) {
          continue;
        }
        let use_symbol = Symbol::new(
          db,
          SymbolKind::Use(*ref_symbol, importing_file, use_node.clone(), name.clone()),
        );
        members.insert(db, use_symbol);
      }
    }
  }
}

/// Recursively collects anonymous element declarations from a block element's body, adding them to the file symbol pool for anonymous types with ImportAnonymous policy
fn collect_nested_anonymous_elements<'db>(
  db: &'db dyn Database,
  file: File,
  block: &BlockElementDeclaration,
  members: &mut StaticScopeMembers<'db>,
) {
  let Some(body) = block.body() else { return };
  for nested in body.element_declarations() {
    let node = nested.syntax();
    // Check if this nested element's type has ImportAnonymous policy
    let nested_as_declaration = ElementDeclaration::cast(node.clone());

    let is_import_anonymous = (|| {
      let typ_symbol =
        (*pretype_resolve_referee(db, file, nested_as_declaration?.typ()?.name()?.node()))?;
      let element_typ = match *pretype_evaluate_typ_symbol(db, typ_symbol) {
        Some(Obj::Typ(Typ::Element(element))) => element,
        _ => return None,
      };
      Some(element_typ.typ_settings(db).on_use == OnUsePolicy::FollowRefs)
    })()
    .unwrap_or(false);

    if is_import_anonymous {
      let declared = declare_element_symbols(db, file, node);
      for symbol in declared {
        members.insert(db, symbol);
      }
    }

    // Recurse into nested block bodies
    collect_nested_anonymous_elements(db, file, &nested, members);
  }
}

/// Returns the (type symbol, label name) pair if the element has a unique label, else None
fn resolve_unique_label<'db>(
  db: &'db dyn Database,
  file: File,
  element: &ElementDeclaration,
) -> Option<(Symbol<'db>, String)> {
  if element.is_typ_declaration() {
    return None;
  }
  let typ_symbol = (*pretype_resolve_referee(db, file, element.typ()?.name()?.node()))?;
  let element_typ = match *pretype_evaluate_typ_symbol(db, typ_symbol) {
    Some(Obj::Typ(Typ::Element(element_typ))) => element_typ,
    _ => return None,
  };
  if !element_typ
    .label_definitions(db)
    .iter()
    .any(|label| label.unique)
  {
    return None;
  }
  let label = element.target_name()?.text().trim().to_string();
  Some((typ_symbol, label))
}
