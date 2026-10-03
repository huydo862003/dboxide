use salsa::{Accumulator, Database, SalsaValue, tracked};

use super::super::typecheck::{get_node_expected_typ, resolve_symbol_typ};
use super::can_namespace_reach::can_namespace_reach_typ;
use crate::ast::{AstNode, CallExpr, IndexExpr, InfixExpr, RedNode};
use crate::db::posttype_analyze::posttype_get_scope_members;
use crate::db::types::interned::symbol::SymbolCategoryKind;
use crate::db::types::{Diagnostics, File, StaticTyp, Symbol, Typ};
use crate::diagnostics::DiagnosticKind;
use crate::syntax::ast::SyntaxKind;
use crate::{
  extract_referee_name, get_node_static_scope, is_namespace_position, posttype_get_symbol_namespace,
};

/// Resolution result for a path fragment (best-effort: symbol may be absent for dynamic items)
#[derive(Clone, Copy, PartialEq, Eq, Hash, SalsaValue)]
pub struct ResolvedReference<'db> {
  pub symbol: Option<Symbol<'db>>,
  pub resolved_typ: Option<Typ<'db>>,
}

/// Narrows chain-path candidates to one resolved ref
#[tracked]
pub fn posttype_resolve_referee<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
) -> Option<ResolvedReference<'db>> {
  let candidates = posttype_resolve_referee_candidates(db, file, node.clone());

  if candidates.is_empty() {
    return None;
  }
  if candidates.len() == 1 {
    return Some(*candidates.first().unwrap());
  }

  // Prefer the candidate whose type can reach the expected type
  let mut picked = None;
  if let Some(Typ::Element(target_typ)) = get_node_expected_typ(db, file, node.clone()).as_ref() {
    for candidate in candidates {
      if let Some(Typ::Element(candidate_typ)) = candidate.resolved_typ
        && *can_namespace_reach_typ(db, candidate_typ, *target_typ)
      {
        picked = Some(*candidate);
        break;
      }
    }
  }
  if picked.is_none() {
    // Multiple candidates remain after disambiguation: warn and pick the first
    Diagnostics::accumulate(
      Diagnostics(
        DiagnosticKind::AmbiguousChain {
          name: chain_terminal_name(&node),
        }
        .into_warning(node.start_offset(), node.end_offset()),
      ),
      db,
    );
  }
  Some(picked.unwrap_or_else(|| *candidates.first().unwrap()))
}

/// Collects every resolution candidate the chain could resolve to
#[tracked]
pub fn posttype_resolve_referee_candidates<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
) -> Vec<ResolvedReference<'db>> {
  // Compound dot expression (a.b.c): the chain resolves to what the RHS resolves to
  if let Some(infix) = InfixExpr::cast(node.clone())
    && infix.has_op(".")
  {
    let Some(rhs) = infix.right() else {
      return Vec::new();
    };
    return posttype_resolve_referee_candidates(db, file, rhs.syntax().clone()).to_vec();
  }

  // Call expression (fn().member): resolve callee return type
  if let Some(call) = CallExpr::cast(node.clone()) {
    if let Some(callee_ref) = *posttype_resolve_referee(db, file, call.syntax().clone())
      && let Some(Typ::Func(func_typ)) = callee_ref.resolved_typ
      && let Some(resolved_typ) = func_typ.ret(db).resolve(db)
    {
      return vec![ResolvedReference {
        symbol: None,
        resolved_typ: Some(resolved_typ),
      }];
    }
    return Vec::new();
  }

  // Index expression (list[i].member): resolve base element/value type
  if let Some(index_expr) = IndexExpr::cast(node.clone()) {
    if let Some(base) = index_expr.base()
      && let Some(base_ref) = *posttype_resolve_referee(db, file, base.syntax().clone())
      && let Some(base_typ) = base_ref.resolved_typ
    {
      let item_typ = match base_typ {
        Typ::List(list_typ) => list_typ.element(db).and_then(|lazy| lazy.resolve(db)),
        Typ::Map(map_typ) => map_typ.value(db).and_then(|lazy| lazy.resolve(db)),
        _ => None,
      };
      if let Some(resolved_typ) = item_typ {
        return vec![ResolvedReference {
          symbol: None,
          resolved_typ: Some(resolved_typ),
        }];
      }
    }
    return Vec::new();
  }

  let Some(name) = extract_referee_name(&node) else {
    return Vec::new();
  };
  let category = infer_expected_category(db, file, &node);

  // RHS of a dot chain: resolve LHS then look up name in its namespace or type
  if let Some(infix) = node.parent().and_then(InfixExpr::cast)
    && infix.has_op(".")
    && infix
      .right()
      .is_some_and(|rhs| rhs.syntax().start_offset() == node.start_offset())
  {
    let Some(lhs) = infix.left() else {
      return Vec::new();
    };

    // Resolve LHS to get all candidate references (symbol + type)
    let lhs_refs = posttype_resolve_referee_candidates(db, file, lhs.syntax().clone());
    let is_ns = is_namespace_position(infix.syntax());

    // For each LHS candidate, try symbol namespace first, then type namespace
    let mut result = Vec::new();
    for lhs_ref in lhs_refs.iter() {
      // Symbol namespace: look up name in the symbol's declared members
      if let Some(lhs_symbol) = lhs_ref.symbol {
        let namespace = posttype_get_symbol_namespace(db, lhs_symbol);
        let symbols = if is_ns {
          namespace
            .lookup_symbols(db, &name, SymbolCategoryKind::Schema)
            .chain(namespace.lookup_symbols(db, &name, SymbolCategoryKind::Element))
            .chain(namespace.lookup_symbols(db, &name, SymbolCategoryKind::Typ))
            .collect::<Vec<_>>()
        } else {
          let mut found: Vec<_> = namespace.lookup_symbols(db, &name, category).collect();
          if found.is_empty() {
            found = namespace
              .lookup_symbols(db, &name, SymbolCategoryKind::Element)
              .chain(namespace.lookup_symbols(db, &name, SymbolCategoryKind::Typ))
              .chain(namespace.lookup_symbols(db, &name, SymbolCategoryKind::Schema))
              .collect();
          }
          found
        };
        for symbol in symbols {
          let reference = ResolvedReference {
            symbol: Some(symbol),
            resolved_typ: resolve_symbol_typ(db, symbol),
          };
          if !result.contains(&reference) {
            result.push(reference);
          }
        }
      }

      // Type namespace fallback: expanded namespace items have no symbol but type is known
      if result.is_empty()
        && let Some(Typ::Element(element_typ)) = lhs_ref.resolved_typ
          && let Some(member_lazy) = element_typ.get_namespace_member(db, &name)
          && let Some(resolved_typ) = member_lazy.resolve(db)
        {
          result.push(ResolvedReference {
            symbol: None,
            resolved_typ: Some(resolved_typ),
          });
        }
    }
    return result;
  }

  // Bare ident: collect across all scope levels without stopping at the first match
  let in_namespace_position = is_namespace_position(&node);
  let mut result = Vec::new();
  let mut current_scope = Some(*get_node_static_scope(db, file, node.clone()));
  while let Some(scope) = current_scope {
    let members = posttype_get_scope_members(db, scope);
    if in_namespace_position {
      for symbol in members
        .lookup_symbols(db, &name, SymbolCategoryKind::Schema)
        .chain(members.lookup_symbols(db, &name, SymbolCategoryKind::Element))
        .chain(members.lookup_symbols(db, &name, category))
      {
        let reference = ResolvedReference {
          symbol: Some(symbol),
          resolved_typ: resolve_symbol_typ(db, symbol),
        };
        if !result.contains(&reference) {
          result.push(reference);
        }
      }
    } else {
      for symbol in members.lookup_symbols(db, &name, category) {
        let reference = ResolvedReference {
          symbol: Some(symbol),
          resolved_typ: resolve_symbol_typ(db, symbol),
        };
        if !result.contains(&reference) {
          result.push(reference);
        }
      }
    }
    current_scope = scope.get_parent(db);
  }
  result
}

/// Extracts the terminal name segment from a potentially compound dot-chain node
fn chain_terminal_name(node: &RedNode) -> String {
  if let Some(infix) = InfixExpr::cast(node.clone())
    && infix.has_op(".")
    && let Some(rhs) = infix.right()
  {
    return chain_terminal_name(rhs.syntax());
  }
  extract_referee_name(node).unwrap_or_default()
}

/// Determine the expected category for a node using type info then syntactic fallback
fn infer_expected_category(db: &dyn Database, file: File, node: &RedNode) -> SymbolCategoryKind {
  if let Some(typ) = *get_node_expected_typ(db, file, node.clone()) {
    return map_typ_to_category(&typ);
  }
  infer_category_syntactic(node)
}

/// Map a type to its primary category
fn map_typ_to_category(typ: &Typ) -> SymbolCategoryKind {
  match typ {
    Typ::Reference(_) | Typ::QualifiedReference(_) => SymbolCategoryKind::Element,
    Typ::TypTyp(_) => SymbolCategoryKind::Typ,
    Typ::Func(_) => SymbolCategoryKind::Function,
    _ => SymbolCategoryKind::Typ,
  }
}

/// Derive category from syntax when no type information is available
fn infer_category_syntactic(node: &RedNode) -> SymbolCategoryKind {
  let mut cursor = node.clone();
  loop {
    let Some(parent) = cursor.parent() else {
      return SymbolCategoryKind::Typ;
    };
    match parent.kind() {
      /* Type-annotation positions */
      SyntaxKind::ElementDeclarationTyp
      | SyntaxKind::FuncDeclarationReturnTyp
      | SyntaxKind::ElementFieldDeclarationArg
      | SyntaxKind::SettingListItemValue => return SymbolCategoryKind::Typ,
      SyntaxKind::UseSpecifierName => return SymbolCategoryKind::Schema,
      /* Dot-chain: walk up through the chain to find the enclosing context */
      SyntaxKind::InfixExpr => {
        cursor = parent;
      }
      /* Bare ident in element body: fallback to Element */
      SyntaxKind::BlockElementDeclarationBody => return SymbolCategoryKind::Element,

      /* Call callee */
      SyntaxKind::CallExpr => {
        if let Some(call) = CallExpr::cast(parent.clone())
          && call
            .callee()
            .is_some_and(|callee| callee.syntax().start_offset() == cursor.start_offset())
        {
          return SymbolCategoryKind::Function;
        }
        cursor = parent;
      }
      _ => cursor = parent,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use itertools::Either;

  use crate::ast::{AstNode, IdentExpr, RedNode, SourceFile};
  use crate::db::DboxideDatabase;
  use crate::db::test_utils::*;
  use crate::db::types::File;
  use crate::db::types::interned::symbol::VirtualModuleKind;
  use crate::parse_file::parse_file;

  use super::posttype_resolve_referee;

  fn parse_root(db: &DboxideDatabase, file: File) -> RedNode {
    parse_file(db, file).ast(db).value(db).node.clone()
  }

  fn find_ident_last(node: &RedNode, target: &str) -> Option<RedNode> {
    let mut result = None;
    if let Some(ident) = IdentExpr::cast(node.clone())
      && ident.name() == target
    {
      result = Some(node.clone());
    }
    for child in node.children() {
      if let Some(found) = find_ident_last(&child, target) {
        result = Some(found);
      }
    }
    result
  }

  fn metatype() -> HashMap<String, Either<VirtualModuleKind, File>> {
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )])
  }

  mod resolution {
    use super::*;

    #[test]
    fn resolution_field_type_ident_resolves_to_declared_typ() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.dbml",
        r#"type Foo {}
type Bar {
  field Foo
}"#,
        HashMap::new(),
      );
      let root = parse_root(&db, file);
      let foo_node = find_ident_last(&root, "Foo").unwrap();
      let result = (*posttype_resolve_referee(&db, file, foo_node)).unwrap();
      assert_eq!(result.symbol.unwrap().name(&db), "Foo");
    }

    #[test]
    fn resolution_element_typ_ident_resolves_to_typ_symbol() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.dbml",
        r#"type Table {}
Table users {}"#,
        HashMap::new(),
      );
      let root = parse_root(&db, file);
      let source_file = SourceFile::cast(root).unwrap();
      let users_decl = source_file
        .element_declarations()
        .find(|declaration| {
          declaration
            .target_name()
            .is_some_and(|fragment| fragment.text().trim() == "users")
        })
        .unwrap();
      let table_node = users_decl.typ().unwrap().name().unwrap().node();
      let result = (*posttype_resolve_referee(&db, file, table_node)).unwrap();
      assert_eq!(result.symbol.unwrap().name(&db), "Table");
    }

    #[test]
    fn resolution_element_ident_in_field_body_resolves_to_element_symbol() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.dbml",
        r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  ref reference[Table] [attribute]
}

Table users {}
Table orders {
  ref: users
}
"#,
        metatype(),
      );
      let root = parse_root(&db, file);
      let users_node = find_ident_last(&root, "users").unwrap();
      let result = (*posttype_resolve_referee(&db, file, users_node)).unwrap();
      assert_eq!(result.symbol.unwrap().name(&db), "users");
    }

    #[test]
    fn resolution_undeclared_ident_returns_none() {
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.dbml",
        r#"type Bar {
  field Missing
}"#,
        HashMap::new(),
      );
      let root = parse_root(&db, file);
      let missing_node = find_ident_last(&root, "Missing").unwrap();
      let result = *posttype_resolve_referee(&db, file, missing_node);
      assert!(result.is_none(), "undeclared ident should not resolve");
    }

    #[test]
    fn resolution_dot_chain_resolves_to_rhs_symbol() {
      // Schema-qualified type reference: `auth.Foo` resolves `Foo` in the `auth` schema namespace
      let db = DboxideDatabase::default();
      let file = create_file(
        &db,
        "test.dbml",
        r#"auth.type Foo {}
type Bar {
  field auth.Foo
}"#,
        HashMap::new(),
      );
      let root = parse_root(&db, file);
      let foo_node = find_ident_last(&root, "Foo").unwrap();
      let result = posttype_resolve_referee(&db, file, foo_node);
      assert!(
        result.is_some(),
        "expected Foo to resolve via schema dot chain"
      );
      assert_eq!(result.as_ref().unwrap().symbol.unwrap().name(&db), "Foo");
    }
  }
}
