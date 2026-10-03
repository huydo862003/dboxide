use std::collections::HashMap;

use salsa::{Accumulator, Database, tracked};

use crate::ast::{
  AstNode, BlockElementDeclaration, BlockElementDeclarationBody, ElementDeclaration, RedNode,
};
use crate::db::types::Diagnostics;
use crate::diagnostics::DiagnosticKind;

use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind, VirtualModuleKind};
use crate::db::types::{
  File, StaticScope, StaticScopeKind, Symbol, get_builtin_members, pretype_get_metatype_members,
};
use crate::pretype_get_file_symbols;

/// Returns scope members visible at a given static scope in pretype context (types and schemas only)
#[tracked]
pub fn pretype_get_scope_members<'db>(
  db: &'db dyn Database,
  scope: StaticScope<'db>,
) -> StaticScopeMembers<'db> {
  match scope.kind(db) {
    StaticScopeKind::Builtin => get_builtin_members(db),
    StaticScopeKind::VirtualModule(VirtualModuleKind::Metatype) => pretype_get_metatype_members(db),
    StaticScopeKind::UserModule(file) => pretype_get_file_symbols(db, *file, false).clone(),
    StaticScopeKind::ElementNamespace(file, node) => {
      pretype_collect_typs_in_element_body(db, *file, node)
    }
    StaticScopeKind::TypNamespace(file, node) => pretype_collect_typs_in_typ_body(db, *file, node),
    StaticScopeKind::Func(..) => StaticScopeMembers::default(),
  }
}

/// Collects type symbols declared directly inside an element body (not a type body)
fn pretype_collect_typs_in_element_body<'db>(
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
  if element.is_typ_declaration() {
    return members;
  }
  let mut seen: HashMap<String, Symbol<'db>> = HashMap::new(); // lowercased type name -> first declared symbol

  /* Nested type declarations inside the element body */
  for nested in body.element_declarations() {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    let Some(name) = element
      .target_name()
      .map(|target| target.text().trim().to_string())
    else {
      continue;
    };
    if !element.is_typ_declaration() {
      continue;
    }
    let node = nested.syntax();
    let symbol = Symbol::new(db, SymbolKind::UserTyp(file, node.clone(), name.clone()));

    if let Some(first_symbol) = seen.get(&name.to_ascii_lowercase()) {
      let SymbolKind::UserTyp(_, first_node, _) = first_symbol.kind(db) else {
        continue;
      };
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::DuplicateTypeDeclaration {
            name,
            first_start: first_node.start_offset(),
            first_end: first_node.end_offset(),
          }
          .into_error(node.start_offset(), node.end_offset()),
        ),
        db,
      );
      continue;
    }
    seen.insert(name.to_ascii_lowercase(), symbol);
    members.insert(db, symbol);
  }

  /* Equality (alias) type declarations inside the element body */
  for equality_declaration in body.equality_declarations() {
    let Some(name_node) = equality_declaration.name() else {
      continue;
    };
    let fragments = name_node.path();
    if fragments.len() != 1 {
      continue;
    }
    let name = fragments.into_iter().next().unwrap().name().to_string();
    let node = equality_declaration.syntax();
    let symbol = Symbol::new(db, SymbolKind::UserTyp(file, node.clone(), name.clone()));
    if let Some(first_symbol) = seen.get(&name.to_ascii_lowercase()) {
      let SymbolKind::UserTyp(_, first_node, _) = first_symbol.kind(db) else {
        continue;
      };
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::DuplicateTypeDeclaration {
            name,
            first_start: first_node.start_offset(),
            first_end: first_node.end_offset(),
          }
          .into_error(node.start_offset(), node.end_offset()),
        ),
        db,
      );
      continue;
    }
    seen.insert(name.to_ascii_lowercase(), symbol);
    members.insert(db, symbol);
  }

  /* Nested element types declared by the parent type are in scope within the element body
  e.g. `indexes { }` is in scope inside `Table users { }` because `type Table` declares `type indexes` */
  let typ_name = element
    .typ()
    .map(|typ| typ.text().trim().to_string())
    .unwrap_or_default();

  // For each matching type symbol: resolve its body, then yield its [element]-tagged nested types
  let inherited = pretype_get_file_symbols(db, file, false)
    .lookup_by_name(&typ_name)
    .filter_map(|(_, symbol)| {
      let SymbolKind::UserTyp(typ_file, definition_node, _) = symbol.original_symbol(db).kind(db)
      else {
        return None;
      };
      let body = BlockElementDeclaration::cast(definition_node.clone())?.body()?;
      Some((*typ_file, body))
    })
    .flat_map(|(typ_file, body)| {
      body
        .element_declarations()
        .filter_map(move |nested_node| {
          let nested_element = ElementDeclaration::cast(nested_node.syntax().clone())?;
          let name = nested_element.target_name()?.text().trim().to_string();
          if !nested_element.is_typ_declaration() || !nested_element.is_nested_element_typ() {
            return None;
          }
          Some((typ_file, nested_node, name))
        })
        .collect::<Vec<_>>()
    });

  for (typ_file, nested, nested_name) in inherited {
    let symbol = Symbol::new(
      db,
      SymbolKind::UserTyp(typ_file, nested.syntax().clone(), nested_name.clone()),
    );
    members.insert(db, symbol);
  }

  members
}

/// Collects type symbols declared directly inside a type body
fn pretype_collect_typs_in_typ_body<'db>(
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
  if !element.is_typ_declaration() {
    return members;
  }
  let mut seen: HashMap<String, Symbol<'db>> = HashMap::new(); // lowercased type name -> first declared symbol

  /* Nested type declarations inside the type body */
  for nested in body.element_declarations() {
    let Some(element) = ElementDeclaration::cast(nested.syntax().clone()) else {
      continue;
    };
    let Some(name) = element
      .target_name()
      .map(|target| target.text().trim().to_string())
    else {
      continue;
    };
    if !element.is_typ_declaration() {
      continue;
    }
    let node = nested.syntax();
    let symbol = Symbol::new(db, SymbolKind::UserTyp(file, node.clone(), name.clone()));
    if let Some(first_symbol) = seen.get(&name.to_ascii_lowercase()) {
      let SymbolKind::UserTyp(_, first_node, _) = first_symbol.kind(db) else {
        continue;
      };
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::DuplicateTypeDeclaration {
            name,
            first_start: first_node.start_offset(),
            first_end: first_node.end_offset(),
          }
          .into_error(node.start_offset(), node.end_offset()),
        ),
        db,
      );
      continue;
    }
    seen.insert(name.to_ascii_lowercase(), symbol);
    members.insert(db, symbol);
  }

  /* Equality (alias) type declarations inside the type body */
  for equality_declaration in body.equality_declarations() {
    let Some(name_node) = equality_declaration.name() else {
      continue;
    };
    let fragments = name_node.path();
    if fragments.len() != 1 {
      continue;
    }
    let name = fragments.into_iter().next().unwrap().name().to_string();
    let node = equality_declaration.syntax();
    let symbol = Symbol::new(db, SymbolKind::UserTyp(file, node.clone(), name.clone()));
    if let Some(first_symbol) = seen.get(&name.to_ascii_lowercase()) {
      let SymbolKind::UserTyp(_, first_node, _) = first_symbol.kind(db) else {
        continue;
      };
      Diagnostics::accumulate(
        Diagnostics(
          DiagnosticKind::DuplicateTypeDeclaration {
            name,
            first_start: first_node.start_offset(),
            first_end: first_node.end_offset(),
          }
          .into_error(node.start_offset(), node.end_offset()),
        ),
        db,
      );
      continue;
    }
    seen.insert(name.to_ascii_lowercase(), symbol);
    members.insert(db, symbol);
  }

  members
}
