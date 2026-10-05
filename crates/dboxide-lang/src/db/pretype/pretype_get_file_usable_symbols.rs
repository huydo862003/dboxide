use std::collections::HashSet;

use itertools::Either;
use salsa::{Database, tracked};

use crate::ast::{AstNode, SourceFile, UseDeclaration};
use crate::parse_file::parse_file;

use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind, VirtualModuleKind};
use crate::db::types::{File, Symbol, pretype_get_metatype_members};

const DEFAULT_SCHEMA: &str = "public";

/// Collect symbols visible from a file
/// reuses_only = true: exported symbols (own + transitive reuse)
/// reuses_only = false: all visible symbols (own + use + reuse)
#[tracked]
pub fn pretype_get_file_usable_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  reuses_only: bool,
) -> StaticScopeMembers<'db> {
  let mut members = StaticScopeMembers::default();
  let mut visited = HashSet::new();
  let mut top_schemas = HashSet::new();
  top_schemas.insert(DEFAULT_SCHEMA.to_string());

  pretype_collect_file_symbols(
    db,
    file,
    reuses_only,
    &mut visited,
    &mut members,
    &mut top_schemas,
  );

  // Register schema symbols for discovered schema names
  let file_path = file.filepath(db).display().to_string();
  for schema_name in &top_schemas {
    if members.lookup_by_name(schema_name).next().is_none() {
      let symbol = Symbol::new(
        db,
        SymbolKind::UserSchema(file, None),
        schema_name.clone(),
        format!("{file_path}::@schema::{schema_name}"),
      );
      members.insert(db, symbol);
    }
  }

  members
}

/// Traverse the import graph collecting types, functions, operators, and discovering schema names
fn pretype_collect_file_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  reuses_only: bool,
  visited: &mut HashSet<File>,
  members: &mut StaticScopeMembers<'db>,
  top_schemas: &mut HashSet<String>,
) {
  if !visited.insert(file) {
    return;
  }

  let file_path = file.filepath(db).display().to_string();
  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return;
  };

  for use_declaration in source_file.use_declarations() {
    if reuses_only && !use_declaration.is_reuse() {
      continue;
    }
    pretype_collect_file_used_symbols(db, file, &use_declaration, visited, members, top_schemas);
  }

  for element_declaration in source_file.element_declarations() {
    let Some(typ) = element_declaration.typ() else {
      continue;
    };

    let (schema_chain, typ_name) = typ.collect_schema_chain();
    top_schemas.insert(schema_chain[0].clone());

    if typ_name != "type" {
      continue;
    }

    let Some(name) = element_declaration.declaration_name() else {
      continue;
    };
    let symbol = Symbol::new(
      db,
      SymbolKind::UserTyp(file, element_declaration.syntax().clone()),
      name.clone(),
      format!("{file_path}::{name}"),
    );
    if schema_chain == [DEFAULT_SCHEMA] {
      members.insert(db, symbol);
    }
  }

  for equality_declaration in source_file.equality_declarations() {
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

  for func_declaration in source_file.func_declarations() {
    let Some(name_node) = func_declaration.name() else {
      continue;
    };
    if name_node.is_operator_declaration() {
      let Some(op_symbol) = name_node.operator_symbol() else {
        continue;
      };
      let symbol = Symbol::new(
        db,
        SymbolKind::UserOperator(file, func_declaration.syntax().clone()),
        op_symbol.clone(),
        format!("{file_path}::operator::{op_symbol}"),
      );
      members.insert(db, symbol);
    } else {
      let name = name_node.text().trim().to_string();
      let symbol = Symbol::new(
        db,
        SymbolKind::UserFunction(file, func_declaration.syntax().clone()),
        name.clone(),
        format!("{file_path}::{name}"),
      );
      members.insert(db, symbol);
    }
  }
}

/// Resolve a use/reuse declaration into Use symbols
fn pretype_collect_file_used_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  use_declaration: &UseDeclaration,
  visited: &mut HashSet<File>,
  members: &mut StaticScopeMembers<'db>,
  top_schemas: &mut HashSet<String>,
) {
  let Some(specifier_path) = use_declaration.module_path() else {
    return;
  };
  let Some(module) = file.uses(db).get(&specifier_path) else {
    return;
  };

  let mut target_symbols = StaticScopeMembers::default();
  match module {
    Either::Left(VirtualModuleKind::Metatype) => {
      target_symbols = pretype_get_metatype_members(db);
    }
    Either::Right(target_file) => {
      pretype_collect_file_symbols(
        db,
        *target_file,
        true,
        visited,
        &mut target_symbols,
        top_schemas,
      );
    }
  }

  // Wildcard: wrap all exported symbols
  if use_declaration.is_wildcard_use() {
    for (_key, original_symbol) in target_symbols.iter() {
      let use_symbol = Symbol::new(
        db,
        SymbolKind::Use(*original_symbol, file, use_declaration.syntax().clone()),
        original_symbol.name(db).to_string(),
        original_symbol.definition_id(db).to_string(),
      );
      members.insert(db, use_symbol);
    }
    return;
  }

  // Selective: match by name, optionally alias
  let Some(specifier_list) = use_declaration.specifier_list() else {
    return;
  };
  for specifier in specifier_list.specifiers() {
    let Some(specifier_name) = specifier.name() else {
      continue;
    };
    let name = specifier_name.text().trim().to_string();
    let matching: Vec<Symbol> = target_symbols
      .lookup_by_name(&name)
      .map(|(_, symbol)| *symbol)
      .collect();
    if matching.is_empty() {
      continue;
    }
    let visible_name = specifier
      .alias()
      .map(|alias_node| alias_node.text().trim().to_string())
      .unwrap_or_else(|| name.clone());
    for original_symbol in matching {
      let use_symbol = Symbol::new(
        db,
        SymbolKind::Use(original_symbol, file, specifier.syntax().clone()),
        visible_name.clone(),
        original_symbol.definition_id(db).to_string(),
      );
      members.insert(db, use_symbol);
    }
  }
}
