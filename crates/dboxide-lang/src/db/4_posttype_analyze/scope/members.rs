use salsa::{Database, tracked};

use crate::ast::{
  AstNode, BlockElementDeclarationBody, ClosureExpr, ElementDeclaration, IdentExpr, RedNode,
  TupleExpr,
};
use crate::posttype_get_metatype_members;

use super::declare::{declare_element_symbols, declare_func_symbol};
use super::imports::posttype_get_file_symbols;
use crate::db::pretype_analyze::pretype_get_scope_members;
use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind, VirtualModuleKind};
use crate::db::types::{File, StaticScope, StaticScopeKind, Symbol, get_builtin_members};

/// Returns scope members visible at a given static scope in posttype context (all symbols)
#[tracked]
pub fn posttype_get_scope_members<'db>(
  db: &'db dyn Database,
  scope: StaticScope<'db>,
) -> StaticScopeMembers<'db> {
  match scope.kind(db) {
    StaticScopeKind::Builtin => get_builtin_members(db),
    StaticScopeKind::VirtualModule(VirtualModuleKind::Metatype) => {
      posttype_get_metatype_members(db)
    }
    StaticScopeKind::UserModule(file) => posttype_get_file_symbols(db, *file, false).clone(),
    StaticScopeKind::ElementNamespace(file, node) => {
      posttype_collect_element_body_symbols(db, *file, node)
    }
    StaticScopeKind::TypNamespace(..) => pretype_get_scope_members(db, scope).clone(), // Types can only be nested in type namespaces
    StaticScopeKind::Func(file, node) => posttype_collect_closure_params(db, *file, node),
  }
}

/// Collects all symbols declared inside an element body (type members, functions, nested elements)
fn posttype_collect_element_body_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::ElementNamespace(file, node.clone()));

  // Start with type members of the element scope
  let mut members = pretype_get_scope_members(db, scope).clone();

  let Some(body) = BlockElementDeclarationBody::cast(node.clone()) else {
    return members;
  };

  /* Scan function declarations */
  for func_declaration in body.func_declarations() {
    if let Some(symbol) = declare_func_symbol(db, file, &func_declaration) {
      members.insert(db, symbol);
    }
  }

  /* Handle nested element declarations - each may declare label/alias symbols */
  for nested in body.element_declarations() {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    if element.is_typ_declaration() {
      continue;
    }
    // Check if element declares symbols
    let declared = declare_element_symbols(db, file, nested.syntax());
    for symbol in declared {
      members.insert(db, symbol);
    }
  }

  members
}

/// Collects closure parameter symbols from a closure expression node
fn posttype_collect_closure_params<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
) -> StaticScopeMembers<'db> {
  let mut members = StaticScopeMembers::default();
  let Some(closure) = ClosureExpr::cast(node.clone()) else {
    return members;
  };

  let Some(params) = closure.params() else {
    return members;
  };

  /* Single param: `x => body` */
  if let Some(ident) = IdentExpr::cast(params.syntax().clone()) {
    let name = ident.name();
    let symbol = Symbol::new(
      db,
      SymbolKind::UserElement(file, params.syntax().clone(), name.clone()),
    );
    members.insert(db, symbol);
  /* Destructured tuple: `(x, y) => body` */
  } else if let Some(tuple) = TupleExpr::cast(params.syntax().clone()) {
    for item in tuple.items() {
      let Some(name) = item.as_name() else {
        continue;
      };
      let symbol = Symbol::new(
        db,
        SymbolKind::UserElement(file, item.syntax().clone(), name.clone()),
      );
      members.insert(db, symbol);
    }
  }

  members
}
