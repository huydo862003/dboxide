mod encode;

pub use encode::*;
use salsa::{Accumulator, Database};

use crate::{
  Diagnostics, ElementTyp, File, StaticScopeMembers, Symbol, SymbolKind, Typ,
  ast::{
    AstNode, DqStringExpr, ElementDeclaration, Expr, IdentExpr, InfixExpr, RedNode, SyntaxKind,
    UseDeclarationTargetFragment, UseSpecifier,
  },
  diagnostics::DiagnosticKind,
  get_declaration_typ,
};

/// Returns true when a typed slot expects a declaration name rather than a value
pub(crate) fn is_declaration_slot(expected_typ: Option<Typ>) -> bool {
  matches!(
    expected_typ,
    Some(Typ::Declaration(_)) | Some(Typ::QualifiedDeclaration(_))
  )
}

/// Extracts the plain string name from an expression in a declaration-typed slot
pub(crate) fn extract_declaration_name(expr: &Expr) -> Option<String> {
  expr.as_name()
}

/// Returns true if the kind specifier uses a built-in keyword kind
pub(crate) fn is_keyword_kind(kind: &UseDeclarationTargetFragment) -> bool {
  kind.name().is_some_and(|fragment| {
    matches!(
      fragment.name().to_lowercase().as_str(),
      "schema" | "type" | "fn" | "operator"
    )
  })
}

/// Extracts the identifier name from a referee node (ident or double-quoted string)
pub(crate) fn extract_referee_name(node: &RedNode) -> Option<String> {
  if let Some(ident) = IdentExpr::cast(node.clone()) {
    return Some(ident.name());
  }
  if let Some(dq) = DqStringExpr::cast(node.clone()) {
    return Some(dq.content());
  }
  None
}

/// Returns true when the node is in a type-annotation position
pub(crate) fn is_in_typ_position(node: &RedNode) -> bool {
  let mut current = node.clone();
  loop {
    let Some(parent) = current.parent() else {
      return false;
    };
    match parent.kind() {
      // Direct type annotation contexts
      SyntaxKind::ElementDeclarationTyp | SyntaxKind::FuncDeclarationReturnTyp => return true,
      // First fragment is always type when not keyword
      SyntaxKind::UseDeclarationTargetFragment => {
        let use_specifier = parent.parent().and_then(UseSpecifier::cast);

        let use_specifier_kind = use_specifier.and_then(|specifier| specifier.kind());

        let Some(use_specifier_kind) = use_specifier_kind else {
          return false;
        };
        if use_specifier_kind.syntax() != &parent {
          return false; // Not in kind position
        }

        if is_keyword_kind(&use_specifier_kind) {
          return false;
        }

        return true;
      }
      // Field type slot: the second arg of a field declaration (arg 0 = name, arg 1 = type)
      SyntaxKind::ElementFieldDeclaration => {
        let mut args = parent
          .children()
          .filter(|child| child.kind() == SyntaxKind::ElementFieldDeclarationArg);
        args.next(); // skip arg 0 (name)
        let type_arg = args.next();
        return type_arg.as_ref().map(|child| child.start_offset()) == Some(current.start_offset());
      }
      // Param type: second child expression (first is the param name)
      SyntaxKind::FuncDeclarationParam => {
        let first_ident = parent
          .children()
          .find(|child| child.kind() == SyntaxKind::IdentExpr)
          .map(|child| child.start_offset());
        return first_ident != Some(current.start_offset());
      }
      // Stop at body/file boundaries
      SyntaxKind::BlockElementDeclarationBody | SyntaxKind::SourceFile => return false,
      _ => {}
    }
    current = parent;
  }
}

/// Returns true when `node` is the LHS of a dot chain (more segments follow)
pub(crate) fn is_namespace_position(node: &RedNode) -> bool {
  node
    .parent()
    .and_then(InfixExpr::cast)
    .is_some_and(|outer| {
      outer.has_op(".")
        && outer
          .left()
          .is_some_and(|lhs| lhs.syntax().start_offset() == node.start_offset())
    })
}

/// Resolves the element type for a symbol, following Use and Alias indirections
pub(crate) fn get_element_typ_for_symbol<'db>(
  db: &'db dyn Database,
  file: File,
  symbol: &Symbol<'db>,
) -> Option<ElementTyp<'db>> {
  let node = match symbol.kind(db) {
    SymbolKind::UserTyp(_, node, _)
    | SymbolKind::UserElement(_, node, _)
    | SymbolKind::AnonymousElement(_, node) => node.clone(),
    SymbolKind::Use(inner, ..) => return get_element_typ_for_symbol(db, file, inner),
    SymbolKind::ElementAlias(inner, _) => return get_element_typ_for_symbol(db, file, inner),
    _ => return None,
  };
  if let Some(Typ::Element(element)) = *get_declaration_typ(db, file, node) {
    Some(element)
  } else {
    None
  }
}

/// Returns true if the symbol represents an element instance (following Use indirections)
pub(crate) fn is_element_symbol(db: &dyn Database, symbol: Symbol) -> bool {
  match symbol.kind(db) {
    SymbolKind::UserElement(..)
    | SymbolKind::AnonymousElement(..)
    | SymbolKind::ElementAlias(..) => true,
    SymbolKind::Use(inner, ..) => is_element_symbol(db, *inner),
    _ => false,
  }
}

/// Returns true if the symbol represents an operator function (following Use indirections)
pub(crate) fn is_operator_symbol(db: &dyn Database, symbol: Symbol) -> bool {
  match symbol.kind(db) {
    SymbolKind::UserOperator(..) => true,
    SymbolKind::Use(inner, ..) => is_operator_symbol(db, *inner),
    _ => false,
  }
}

/// Returns true if the use symbol can be inserted - false if it conflicts or is already present
pub(crate) fn check_import_collision<'db>(
  db: &'db dyn Database,
  members: &StaticScopeMembers<'db>,
  visible_name: &str,
  original: Symbol<'db>,
  error_start: usize,
  error_end: usize,
) -> bool {
  let new_original = original.original_symbol(db);
  let mut found_local = false;
  let mut found_conflicting_use = false;

  for (_, existing) in members.lookup_by_name(visible_name) {
    if let SymbolKind::Use(existing_inner, _, _, _) = existing.kind(db) {
      /* Same underlying symbol - already imported, skip silently */
      if existing_inner.original_symbol(db) == new_original {
        return false;
      }
      found_conflicting_use = true;
    } else {
      found_local = true;
    }
  }

  if found_local {
    /* Local declaration with the same name takes precedence - import is not allowed */
    Diagnostics::accumulate(
      Diagnostics(
        DiagnosticKind::ImportConflictsWithLocal {
          name: visible_name.to_string(),
        }
        .into_error(error_start, error_end),
      ),
      db,
    );
    return false;
  }
  if found_conflicting_use {
    /* Different symbol already imported under the same name */
    Diagnostics::accumulate(
      Diagnostics(
        DiagnosticKind::ConflictingImport {
          name: visible_name.to_string(),
        }
        .into_error(error_start, error_end),
      ),
      db,
    );
    return false;
  }
  true
}

/// Returns true if `symbol` matches the kind specifier fragment
/// Handles keyword kinds (`schema`, `type`, `fn`, `operator`) and user-defined type kinds
/// Uses pretype resolution for the kind name to avoid re-entering posttype scope queries
pub(crate) fn does_symbol_match_kind<'db>(
  db: &'db dyn Database,
  file: File,
  kind_fragment: &UseDeclarationTargetFragment,
  symbol: Symbol<'db>,
) -> bool {
  use crate::pretype_resolve_referee;

  let Some(kind_name) = kind_fragment
    .name()
    .map(|name_node| name_node.name().to_lowercase())
  else {
    return false;
  };

  // Special keyword kind imports
  match kind_name.as_str() {
    "schema" => return matches!(symbol.kind(db), SymbolKind::UserSchema(..)),
    "type" => {
      return matches!(
        symbol.kind(db),
        SymbolKind::UserTyp(..) | SymbolKind::VirtualTyp(..)
      );
    }
    "fn" => {
      return matches!(
        symbol.kind(db),
        SymbolKind::UserFunction(..) | SymbolKind::VirtualFunction(..)
      );
    }
    "operator" => return matches!(symbol.kind(db), SymbolKind::UserOperator(..)),
    _ => {}
  }

  // User-defined kind: resolve the kind name as a type using pretype only
  let Some(resolved_kind) = kind_fragment
    .name()
    .and_then(|name_node| *pretype_resolve_referee(db, file, name_node.node()))
    .map(|kind| kind.original_symbol(db))
  else {
    return false;
  };

  let inner = symbol.original_symbol(db);
  if let SymbolKind::UserElement(element_file, node, _) = inner.kind(db)
    && let Some(element) = ElementDeclaration::cast(node.clone())
    && let Some(typ_name) = element.typ().and_then(|typ| typ.name())
    && let Some(element_kind_symbol) = pretype_resolve_referee(db, *element_file, typ_name.node())
  {
    return element_kind_symbol.original_symbol(db) == resolved_kind;
  }
  false
}
