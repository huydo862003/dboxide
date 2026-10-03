mod schema;
use schema::collect_usable_members_for_schema;

#[cfg(test)]
mod tests;

use salsa::{Database, tracked};

use crate::ast::{AstNode, BlockElementDeclaration, ElementDeclaration, RedNode};
use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind};
use crate::db::types::{File, Symbol};

/// Returns the namespace members for a symbol in pretype context (schemas/types only)
#[tracked]
pub fn pretype_get_symbol_namespace<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> StaticScopeMembers<'db> {
  match symbol.kind(db) {
    SymbolKind::UserSchema(file, _, _) => collect_usable_members_for_schema(db, *file, symbol),
    SymbolKind::UserTyp(file, node, _) => collect_nested_typ_members(db, *file, node),
    SymbolKind::Use(inner, ..) => pretype_get_symbol_namespace(db, *inner).clone(),
    _ => StaticScopeMembers::default(),
  }
}

/// Collects nested type declarations from a type body as namespace members
fn collect_nested_typ_members<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
) -> StaticScopeMembers<'db> {
  let mut members = StaticScopeMembers::default();
  let Some(block) = BlockElementDeclaration::cast(node.clone()) else {
    return members;
  };
  let Some(body) = block.body() else {
    return members;
  };
  for nested in body.element_declarations() {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    if !element.is_typ_declaration() {
      continue;
    }
    let Some(name) = element
      .target_name()
      .map(|target| target.text().trim().to_string())
    else {
      continue;
    };
    members.insert(
      db,
      Symbol::new(
        db,
        SymbolKind::UserTyp(file, nested.syntax().clone(), name.clone()),
      ),
    );
  }
  members
}
