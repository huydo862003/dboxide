use std::collections::HashSet;

use itertools::Either;
use salsa::Database;

use crate::ast::{AstNode, SourceFile};
use crate::parse_file::parse_file;

use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind};
use crate::db::types::{File, Symbol};

/// Collect usable members for a schema
///
/// Schema members come from the owning file's own declarations only.
/// Child schema names are collected globally across all reachable files and selective schema imports.
pub(super) fn collect_usable_members_for_schema<'db>(
  db: &'db dyn Database,
  file: File,
  schema_qualified_name: &[String],
) -> StaticScopeMembers<'db> {
  let file_path = file.filepath(db).display().to_string();
  let mut members = StaticScopeMembers::default();

  // Collect direct members from the owning file only
  collect_own_schema_members(db, file, schema_qualified_name, &mut members);

  // Collect child schema names globally across all reachable files
  let child_schemas = collect_child_schema_names(db, file, schema_qualified_name);
  let parent_symbol = build_schema_chain(db, file, &file_path, schema_qualified_name);
  for child_name in child_schemas {
    let mut child_qualified = schema_qualified_name.to_vec();
    child_qualified.push(child_name.clone());
    let child_symbol = Symbol::new(
      db,
      SymbolKind::UserSchema(file, parent_symbol),
      child_name.clone(),
      format!("{file_path}::@schema::{}", child_qualified.join(".")),
    );
    members.insert(db, child_symbol);
  }

  members
}

/// Only types are collected in pretype, not elements
fn collect_own_schema_members<'db>(
  db: &'db dyn Database,
  file: File,
  schema_qualified_name: &[String],
  members: &mut StaticScopeMembers<'db>,
) {
  let file_path = file.filepath(db).display().to_string();
  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return;
  };

  for element_declaration in source_file.element_declarations() {
    let Some(typ) = element_declaration.typ() else {
      continue;
    };
    let Some(name) = element_declaration.declaration_name() else {
      continue;
    };

    let (schema_chain, typ_text) = typ.collect_schema_chain();

    if schema_chain != schema_qualified_name {
      continue;
    }

    if typ_text != "type" {
      continue;
    }

    let symbol = Symbol::new(
      db,
      SymbolKind::UserTyp(file, element_declaration.syntax().clone()),
      name.clone(),
      format!("{file_path}::{name}"),
    );
    members.insert(db, symbol);
  }
}

/// Schema names are global: every file in the import graph contributes schema names
fn collect_child_schema_names(
  db: &dyn Database,
  file: File,
  parent_qualified_name: &[String],
) -> HashSet<String> {
  let mut child_names = HashSet::new();
  let mut visited = HashSet::new();
  collect_child_schema_names_recursive(
    db,
    file,
    parent_qualified_name,
    &mut visited,
    &mut child_names,
  );
  child_names
}

fn collect_child_schema_names_recursive(
  db: &dyn Database,
  file: File,
  parent_qualified_name: &[String],
  visited: &mut HashSet<File>,
  child_names: &mut HashSet<String>,
) {
  if !visited.insert(file) {
    return;
  }

  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return;
  };

  for element_declaration in source_file.element_declarations() {
    let Some(typ) = element_declaration.typ() else {
      continue;
    };
    let (schema_chain, _) = typ.collect_schema_chain();
    if schema_chain.len() > parent_qualified_name.len()
      && parent_qualified_name
        .iter()
        .zip(schema_chain.iter())
        .all(|(expected, actual)| expected == actual)
    {
      child_names.insert(schema_chain[parent_qualified_name.len()].clone());
    }
  }

  let uses = file.uses(db);
  for use_declaration in source_file.use_declarations() {
    // `use { schema x.a }` registers child schema "a" under "x"
    if let Some(specifier_list) = use_declaration.specifier_list() {
      for specifier in specifier_list.specifiers() {
        let is_schema = specifier
          .kind()
          .is_some_and(|k| k.text().trim().eq_ignore_ascii_case("schema"));
        if !is_schema {
          continue;
        }
        let Some(name_node) = specifier.name() else {
          continue;
        };
        let parts: Vec<String> = name_node
          .path_fragments()
          .map(|f| f.text().trim().to_string())
          .collect();
        if parts.len() > parent_qualified_name.len()
          && parent_qualified_name
            .iter()
            .zip(parts.iter())
            .all(|(expected, actual)| expected == actual)
        {
          child_names.insert(parts[parent_qualified_name.len()].clone());
        }
      }
    }

    // Follow dep files to discover schema names globally
    let Some(specifier) = use_declaration.module_path() else {
      continue;
    };
    if let Some(Either::Right(dep_file)) = uses.get(&specifier) {
      collect_child_schema_names_recursive(
        db,
        *dep_file,
        parent_qualified_name,
        visited,
        child_names,
      );
    }
  }
}

/// e.g. `["auth", "sub"]` -> UserSchema("sub", parent=UserSchema("auth", parent=None))
fn build_schema_chain<'db>(
  db: &'db dyn Database,
  file: File,
  file_path: &str,
  qualified_name: &[String],
) -> Option<Symbol<'db>> {
  if qualified_name.is_empty() {
    return None;
  }
  let mut current_parent: Option<Symbol<'db>> = None;
  for (index, segment) in qualified_name.iter().enumerate() {
    let definition_id = format!(
      "{file_path}::@schema::{}",
      qualified_name[..=index].join(".")
    );
    let symbol = Symbol::new(
      db,
      SymbolKind::UserSchema(file, current_parent),
      segment.clone(),
      definition_id,
    );
    current_parent = Some(symbol);
  }
  current_parent
}
