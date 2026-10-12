mod schema;
use schema::collect_usable_members_for_schema;

use std::collections::BTreeMap;

use salsa::{Database, tracked};

use crate::ast::{AstNode, BlockElementDeclaration, ElementFieldDeclaration, RedNode};
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind};
use crate::db::types::{File, Symbol};

use crate::db::posttype::resolve::resolve_element_typ::resolve_element_typ;
use crate::db::posttype::scope::declare_element::declare_element_symbols;

// Pretype: schemas only
#[tracked]
pub fn get_symbol_namespace_pretype<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> StaticScopeMembers<'db> {
  match symbol.kind(db) {
    SymbolKind::UserSchema(file, parent) => {
      let qualified_name = build_schema_qualified_name(db, symbol, parent);
      collect_usable_members_for_schema(db, *file, &qualified_name)
    }
    _ => StaticScopeMembers::default(),
  }
}

// Posttype: extends pretype namespace with element instance children
pub fn get_symbol_namespace_posttype<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> StaticScopeMembers<'db> {
  match symbol.kind(db) {
    SymbolKind::UserElement(file, node) => collect_element_namespace(db, *file, node),
    SymbolKind::ElementAlias(primary) => get_symbol_namespace_posttype(db, *primary),
    _ => get_symbol_namespace_pretype(db, symbol).clone(),
  }
}

fn collect_element_namespace<'db>(
  db: &'db dyn Database,
  file: File,
  element_node: &RedNode,
) -> StaticScopeMembers<'db> {
  let mut members = StaticScopeMembers::default();

  let Some(element_typ) = resolve_element_typ(db, file, element_node) else {
    return members;
  };

  let Some(block) = BlockElementDeclaration::cast(element_node.clone()) else {
    return members;
  };
  let Some(body) = block.body() else {
    return members;
  };

  for nested in body.element_declarations() {
    let declared = declare_element_symbols(db, file, nested.syntax());
    for symbol in declared {
      members.insert(db, symbol);
    }
  }

  let field_subtypes = element_typ.field_subtypes(db);
  if !field_subtypes.is_empty() {
    for field in body.fields() {
      if let Some(symbol) = declare_field_element_symbols(db, file, &field, field_subtypes) {
        members.insert(db, symbol);
      }
    }
  }

  members
}

fn declare_field_element_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  field: &ElementFieldDeclaration,
  field_subtypes: &BTreeMap<String, LazyTyp<'db>>,
) -> Option<Symbol<'db>> {
  let args: Vec<_> = field.args().collect();
  if args.len() < 2 {
    return None;
  }

  let first_text = args[0].text().trim().to_string();
  let matches_subtype = field_subtypes
    .keys()
    .any(|k| k.eq_ignore_ascii_case(&first_text));
  if !matches_subtype {
    return None;
  }

  let name = args[1].text().trim().to_string();
  if name.is_empty() {
    return None;
  }

  let file_path = file.filepath(db).display().to_string();
  Some(Symbol::new(
    db,
    SymbolKind::UserElement(file, field.syntax().clone()),
    name.clone(),
    format!("{file_path}::{name}"),
  ))
}

fn build_schema_qualified_name<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
  parent: &Option<Symbol<'db>>,
) -> Vec<String> {
  let mut chain = vec![symbol.name(db).to_string()];
  let mut current_parent = *parent;
  while let Some(parent_symbol) = current_parent {
    chain.push(parent_symbol.name(db).to_string());
    current_parent = match parent_symbol.kind(db) {
      SymbolKind::UserSchema(_, parent_schema) => *parent_schema,
      _ => None,
    };
  }
  chain.reverse();
  chain
}
