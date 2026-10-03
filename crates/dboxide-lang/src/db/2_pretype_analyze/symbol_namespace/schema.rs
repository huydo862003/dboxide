use std::collections::HashSet;

use itertools::Either;
use salsa::Database;

use crate::ast::{AstNode, SourceFile};
use crate::parse_file::parse_file;

use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind};
use crate::db::types::{File, Symbol};

/// Collect usable members for a schema
///
/// Schema members are merged across the owning file and all reachable dep files
/// Child schema names are collected globally across all reachable files and selective schema imports
pub(super) fn collect_usable_members_for_schema<'db>(
  db: &'db dyn Database,
  file: File,
  schema_symbol: Symbol<'db>,
) -> StaticScopeMembers<'db> {
  let schema_qualified_name = schema_symbol.schema_qualified_name(db);
  let mut members = StaticScopeMembers::default();

  // Collect members from the owning file and all reachable dep files (merged)
  let mut visited = HashSet::new();
  collect_schema_members_recursive(db, file, &schema_qualified_name, &mut visited, &mut members);

  // Collect child schema names globally across all reachable files
  let child_schemas = collect_child_schema_names(db, file, &schema_qualified_name);
  for child_name in child_schemas {
    let child_symbol = Symbol::new(
      db,
      SymbolKind::UserSchema(file, Some(schema_symbol), child_name.clone()),
    );
    members.insert(db, child_symbol);
  }

  members
}

/// Recursively collect schema members from a file and all its reachable deps
fn collect_schema_members_recursive<'db>(
  db: &'db dyn Database,
  file: File,
  schema_qualified_name: &[String],
  visited: &mut HashSet<File>,
  members: &mut StaticScopeMembers<'db>,
) {
  if !visited.insert(file) {
    return;
  }
  collect_own_schema_members(db, file, schema_qualified_name, members);

  // Follow all imports to merge schema members (schemas are global)
  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return;
  };
  let uses = file.uses(db);
  for use_declaration in source_file.use_declarations() {
    let Some(specifier) = use_declaration.module_path() else {
      continue;
    };
    if let Some(Either::Right(dep_file)) = uses.get(&specifier) {
      collect_schema_members_recursive(db, *dep_file, schema_qualified_name, visited, members);
    }
  }
}

/// Only types are collected in pretype, not elements
fn collect_own_schema_members<'db>(
  db: &'db dyn Database,
  file: File,
  schema_qualified_name: &[String],
  members: &mut StaticScopeMembers<'db>,
) {
  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return;
  };

  for element_declaration in source_file.element_declarations() {
    let Some(typ) = element_declaration.typ() else {
      continue;
    };

    // Schema-qualified type declarations: `auth.type Foo {}`
    // own_name is "type" and schema_chain matches the requested schema
    if !typ
      .name()
      .is_some_and(|typ_name| typ_name.name().eq_ignore_ascii_case("type"))
    {
      continue;
    }

    if !typ
      .schema()
      .iter()
      .map(|schema_fragment| schema_fragment.name())
      .eq(schema_qualified_name.iter().map(|schema| schema.as_str()))
    {
      continue;
    }

    let Some(name) = element_declaration
      .target_name()
      .and_then(|target| target.name())
      .map(|fragment| fragment.name().to_string())
    else {
      continue;
    };

    let symbol = Symbol::new(
      db,
      SymbolKind::UserTyp(file, element_declaration.syntax().clone(), name.clone()),
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

/// Recursively collects child schema names reachable from a file and its imports
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
    let schema_chain = typ.schema();
    if schema_chain.len() > parent_qualified_name.len()
      && parent_qualified_name
        .iter()
        .zip(schema_chain.iter())
        .all(|(expected, actual)| expected.as_str() == actual.name())
    {
      child_names.insert(schema_chain[parent_qualified_name.len()].name().to_string());
    }
  }

  let uses = file.uses(db);
  for use_declaration in source_file.use_declarations() {
    // `use { schema x.a }` registers child schema "a" under "x"
    if let Some(specifier_list) = use_declaration.specifier_list() {
      for specifier in specifier_list.specifiers() {
        let is_schema = specifier.kind().is_some_and(|kind| {
          kind
            .name()
            .is_some_and(|fragment| fragment.name().eq_ignore_ascii_case("schema"))
        });
        if !is_schema {
          continue;
        }
        let Some(name_node) = specifier.name() else {
          continue;
        };
        let parts: Vec<String> = name_node
          .path()
          .into_iter()
          .map(|fragment| fragment.name().to_string())
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
