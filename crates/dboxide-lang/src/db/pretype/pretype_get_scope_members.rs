use salsa::{Database, tracked};

use crate::ast::{AstNode, BlockElementDeclarationBody, ElementDeclaration, RedNode};

use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind, VirtualModuleKind};
use crate::db::types::{File, StaticScope, StaticScopeKind, Symbol, pretype_get_metatype_members};

use super::pretype_get_file_usable_symbols::get_file_types_and_schemas;

#[tracked]
pub fn get_scope_types_and_schemas<'db>(
  db: &'db dyn Database,
  scope: StaticScope<'db>,
) -> StaticScopeMembers<'db> {
  match scope.kind(db) {
    StaticScopeKind::Builtin => StaticScopeMembers::default(),
    StaticScopeKind::VirtualModule(VirtualModuleKind::Metatype) => pretype_get_metatype_members(db),
    StaticScopeKind::UserModule(file) => get_file_types_and_schemas(db, *file, false).clone(),
    StaticScopeKind::ElementNamespace(file, node) => collect_types_in_element_body(db, *file, node),
    StaticScopeKind::TypNamespace(file, node) => collect_types_in_typ_body(db, *file, node),
    StaticScopeKind::Func(..) => StaticScopeMembers::default(),
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
