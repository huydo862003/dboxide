use salsa::{Database, tracked};

use crate::ast::{
  AstNode, BlockElementDeclarationBody, ClosureExpr, ElementDeclaration, IdentExpr, RedNode,
  TupleExpr,
};

use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind, VirtualModuleKind};
use crate::db::types::{File, StaticScope, StaticScopeKind, Symbol, pretype_get_metatype_members};

use super::pretype_get_file_usable_symbols::pretype_get_file_usable_symbols;

/// Get all members visible at a given scope level
/// - In builtins and modules, only type declaration, function declarations, getter declarations are returned (elements won't be returned here)
/// - In block/func declarations, local types, functions can be returned, and element symbols can be returned for signature
#[tracked]
pub fn pretype_get_scope_members<'db>(
  db: &'db dyn Database,
  scope: StaticScope<'db>,
) -> StaticScopeMembers<'db> {
  match scope.kind(db) {
    StaticScopeKind::Builtin => StaticScopeMembers::default(),
    StaticScopeKind::VirtualModule(VirtualModuleKind::Metatype) => pretype_get_metatype_members(db),
    StaticScopeKind::UserModule(file) => pretype_get_file_usable_symbols(db, *file, false).clone(),
    StaticScopeKind::ElementNamespace(file, node) => {
      collect_node_symbols_in_element_body(db, *file, node)
    }
    StaticScopeKind::TypNamespace(file, node) => collect_node_symbols_in_typ_body(db, *file, node),
    StaticScopeKind::Func(file, node) => pretype_collect_closure_scope_symbols(db, *file, node),
  }
}

fn pretype_collect_closure_scope_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
) -> StaticScopeMembers<'db> {
  let mut members = StaticScopeMembers::default();
  let Some(closure) = ClosureExpr::cast(node.clone()) else {
    return members;
  };
  let file_path = file.filepath(db).display().to_string();

  let Some(params) = closure.params() else {
    return members;
  };

  if let Some(ident) = IdentExpr::cast(params.syntax().clone()) {
    let name = ident.name();
    let symbol = Symbol::new(
      db,
      SymbolKind::UserElement(file, params.syntax().clone()),
      name.clone(),
      format!("{file_path}::@param::{name}"),
    );
    members.insert(db, symbol);
  } else if let Some(tuple) = TupleExpr::cast(params.syntax().clone()) {
    for item in tuple.items() {
      let Some(name) = item.as_name() else {
        continue;
      };
      let symbol = Symbol::new(
        db,
        SymbolKind::UserElement(file, item.syntax().clone()),
        name.clone(),
        format!("{file_path}::@param::{name}"),
      );
      members.insert(db, symbol);
    }
  }

  members
}

/// Collect types and functions declared inside a block body
pub(super) fn collect_node_symbols_in_element_body<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
) -> StaticScopeMembers<'db> {
  let mut members = StaticScopeMembers::default();
  let Some(body) = BlockElementDeclarationBody::cast(node.clone()) else {
    return members;
  };

  let Some(element) = node
    .parent()
    .and_then(|element_node| ElementDeclaration::cast(element_node.clone()))
  else {
    return members;
  };
  let Some(typ) = element.typ() else {
    return members;
  };
  if typ.text().trim().to_lowercase() == "type" {
    return members;
  }

  let file_path = file.filepath(db).display().to_string();

  for nested in body.element_declarations() {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    let Some(name) = element.declaration_name() else {
      continue;
    };
    let Some(typ) = element.typ() else {
      continue;
    };
    if typ.text().trim().to_lowercase() != "type" {
      continue;
    }
    let symbol = Symbol::new(
      db,
      SymbolKind::UserTyp(file, nested.syntax().clone()),
      name.clone(),
      format!("{file_path}::{name}"),
    );
    members.insert(db, symbol);
  }

  for equality_declaration in body.equality_declarations() {
    let Some(name_node) = equality_declaration.name() else {
      continue;
    };
    let name = name_node.text().trim().to_string();
    let symbol = Symbol::new(
      db,
      SymbolKind::UserTyp(file, equality_declaration.syntax().clone()),
      name.clone(),
      format!("{file_path}::{name}"),
    );
    members.insert(db, symbol);
  }

  for fn_declaration in body.func_declarations() {
    let Some(name_node) = fn_declaration.name() else {
      continue;
    };
    if name_node.is_operator_declaration() {
      let Some(op_symbol) = name_node.operator_symbol() else {
        continue;
      };
      let symbol = Symbol::new(
        db,
        SymbolKind::UserOperator(file, fn_declaration.syntax().clone()),
        op_symbol.clone(),
        format!("{file_path}::operator::{op_symbol}"),
      );
      members.insert(db, symbol);
    } else {
      let name = name_node.text().trim().to_string();
      let symbol = Symbol::new(
        db,
        SymbolKind::UserFunction(file, fn_declaration.syntax().clone()),
        name.clone(),
        format!("{file_path}::{name}"),
      );
      members.insert(db, symbol);
    }
  }

  members
}

/// Collect types and functions declared inside a block body
pub(super) fn collect_node_symbols_in_typ_body<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
) -> StaticScopeMembers<'db> {
  let mut members = StaticScopeMembers::default();

  let Some(body) = BlockElementDeclarationBody::cast(node.clone()) else {
    return members;
  };

  let Some(element) = node
    .parent()
    .and_then(|element_node| ElementDeclaration::cast(element_node.clone()))
  else {
    return members;
  };
  let Some(typ) = element.typ() else {
    return members;
  };
  if typ.text().trim().to_lowercase() != "type" {
    return members;
  }

  let file_path = file.filepath(db).display().to_string();

  for nested in body.element_declarations() {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    let Some(name) = element.declaration_name() else {
      continue;
    };
    let Some(typ) = element.typ() else {
      continue;
    };
    if typ.text().trim().to_lowercase() != "type" {
      continue;
    }
    let symbol = Symbol::new(
      db,
      SymbolKind::UserTyp(file, nested.syntax().clone()),
      name.clone(),
      format!("{file_path}::{name}"),
    );
    members.insert(db, symbol);
  }

  for equality_declaration in body.equality_declarations() {
    let Some(name_node) = equality_declaration.name() else {
      continue;
    };
    let name = name_node.text().trim().to_string();
    let symbol = Symbol::new(
      db,
      SymbolKind::UserTyp(file, equality_declaration.syntax().clone()),
      name.clone(),
      format!("{file_path}::{name}"),
    );
    members.insert(db, symbol);
  }

  for fn_declaration in body.func_declarations() {
    let Some(name_node) = fn_declaration.name() else {
      continue;
    };
    if name_node.is_operator_declaration() {
      let Some(op_symbol) = name_node.operator_symbol() else {
        continue;
      };
      let symbol = Symbol::new(
        db,
        SymbolKind::UserOperator(file, fn_declaration.syntax().clone()),
        op_symbol.clone(),
        format!("{file_path}::operator::{op_symbol}"),
      );
      members.insert(db, symbol);
    } else {
      let name = name_node.text().trim().to_string();
      let symbol = Symbol::new(
        db,
        SymbolKind::UserFunction(file, fn_declaration.syntax().clone()),
        name.clone(),
        format!("{file_path}::{name}"),
      );
      members.insert(db, symbol);
    }
  }

  members
}
