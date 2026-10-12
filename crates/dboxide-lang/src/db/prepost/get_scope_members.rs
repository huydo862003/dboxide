use salsa::{Database, tracked};

use crate::ast::{
  AstNode, BlockElementDeclarationBody, ClosureExpr, ElementDeclaration, IdentExpr, RedNode,
  TupleExpr,
};

use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind, VirtualModuleKind};
use crate::db::types::{
  File, StaticScope, StaticScopeKind, Symbol, get_metatype_types_and_schemas,
};

use super::get_file_symbols::{get_file_symbols_posttype, get_file_symbols_pretype};
use crate::db::posttype::scope::declare_element::declare_element_symbols;

// Pretype: types and schemas only
#[tracked]
pub fn get_scope_members_pretype<'db>(
  db: &'db dyn Database,
  scope: StaticScope<'db>,
) -> StaticScopeMembers<'db> {
  match scope.kind(db) {
    StaticScopeKind::Builtin => StaticScopeMembers::default(),
    StaticScopeKind::VirtualModule(VirtualModuleKind::Metatype) => {
      get_metatype_types_and_schemas(db)
    }
    StaticScopeKind::UserModule(file) => get_file_symbols_pretype(db, *file, false).clone(),
    StaticScopeKind::ElementNamespace(file, node) => collect_types_in_element_body(db, *file, node),
    StaticScopeKind::TypNamespace(file, node) => collect_types_in_typ_body(db, *file, node),
    StaticScopeKind::Func(..) => StaticScopeMembers::default(),
  }
}

// Posttype: extends pretype scope with functions, operators, element instances, and closure params
#[tracked]
pub fn get_scope_members_posttype<'db>(
  db: &'db dyn Database,
  scope: StaticScope<'db>,
) -> StaticScopeMembers<'db> {
  match scope.kind(db) {
    StaticScopeKind::Builtin => StaticScopeMembers::default(),
    StaticScopeKind::VirtualModule(VirtualModuleKind::Metatype) => {
      get_metatype_types_and_schemas(db)
    }
    StaticScopeKind::UserModule(file) => get_file_symbols_posttype(db, *file, false).clone(),
    StaticScopeKind::ElementNamespace(file, node) => collect_element_body_symbols(db, *file, node),
    StaticScopeKind::TypNamespace(..) => get_scope_members_pretype(db, scope).clone(),
    StaticScopeKind::Func(file, node) => collect_closure_params(db, *file, node),
  }
}

fn collect_types_in_element_body<'db>(
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

  members
}

fn collect_types_in_typ_body<'db>(
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

  members
}

fn collect_element_body_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  node: &RedNode,
) -> StaticScopeMembers<'db> {
  let scope = StaticScope::new(db, StaticScopeKind::ElementNamespace(file, node.clone()));
  let mut members = get_scope_members_pretype(db, scope).clone();

  let Some(body) = BlockElementDeclarationBody::cast(node.clone()) else {
    return members;
  };

  let file_path = file.filepath(db).display().to_string();

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

  for nested in body.element_declarations() {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    let Some(typ) = element.typ() else {
      continue;
    };
    if typ.text().trim().to_lowercase() == "type" {
      continue;
    }
    let declared = declare_element_symbols(db, file, nested.syntax());
    for symbol in declared {
      members.insert(db, symbol);
    }
  }

  members
}

fn collect_closure_params<'db>(
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
