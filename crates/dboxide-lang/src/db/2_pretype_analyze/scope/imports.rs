use std::collections::{HashMap, HashSet};

use itertools::Either;
use salsa::{Accumulator, Database, tracked};

use crate::DEFAULT_SCHEMA;
use crate::ast::{AstNode, SourceFile, UseDeclaration};
use crate::db::types::Diagnostics;
use crate::diagnostics::DiagnosticKind;
use crate::parse_file::parse_file;

use crate::db::types::interned::symbol::{StaticScopeMembers, SymbolKind, VirtualModuleKind};
use crate::db::types::{
  File, Obj, OnUsePolicy, StaticTyp, Symbol, Typ, pretype_get_metatype_members,
};

use crate::db::pretype_analyze::get_declaration_typ;
use crate::db::pretype_evaluate_typ::pretype_evaluate_typ_symbol;
use crate::utils::{check_import_collision, does_symbol_match_kind, is_keyword_kind};

/// Collects type and schema symbols visible in a file at pretype scope
/// - `reuses_only = true`: exported symbols (own + transitive reuse)
/// - `reuses_only = false`: all visible symbols (own + use + reuse)
#[tracked]
pub fn pretype_get_file_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  reuses_only: bool,
) -> StaticScopeMembers<'db> {
  let mut members_result = StaticScopeMembers::default();
  let mut encountered_top_schemas = HashSet::from([DEFAULT_SCHEMA.to_string()]);

  pretype_collect_file_symbols(
    db,
    file,
    reuses_only,
    &mut members_result,
    &mut encountered_top_schemas,
  );

  for schema_name in &encountered_top_schemas {
    let symbol = Symbol::new(db, SymbolKind::UserSchema(file, None, schema_name.clone()));
    members_result.insert(db, symbol);
  }

  members_result
}

/// Collects all type and schema symbols from a file into `members`, merging with existing state
pub(crate) fn pretype_collect_file_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  reuses_only: bool,
  members: &mut StaticScopeMembers<'db>,
  top_schemas: &mut HashSet<String>,
) {
  let root_node = parse_file(db, file).ast(db).value(db).node.clone();
  let Some(source_file) = SourceFile::cast(root_node) else {
    return;
  };

  let mut seen: HashMap<String, Symbol<'db>> = HashMap::new(); // lowercased type name -> first declared symbol

  /*
   * Collect schemas referenced anywhere in the file (for top-level schema tracking)
   * Collect block-form type declarations (`type Foo { ... }`)
   * */
  for element_declaration in source_file.element_declarations() {
    /* Track any schema qualifiers on typ, kind, or target */
    if let Some(typ) = element_declaration.typ()
      && let Some(schema) = typ.schema().into_iter().next()
    {
      top_schemas.insert(schema.name().to_string());
    }
    if let Some(kind) = element_declaration.target_kind()
      && let Some(schema) = kind.schema().into_iter().next()
    {
      top_schemas.insert(schema.name().to_string());
    }
    if let Some(target) = element_declaration.target_name()
      && let Some(schema) = target.schema().into_iter().next()
    {
      top_schemas.insert(schema.name().to_string());
    }

    /* Only type declarations contribute to the type namespace */
    if !element_declaration.is_typ_declaration() {
      continue;
    }

    /* Must resolve to a TypTyp (not an element instance) */
    let Some(Typ::TypTyp(_)) = get_declaration_typ(db, file, element_declaration.syntax().clone())
    else {
      continue;
    };

    let Some(target) = element_declaration.target_name() else {
      continue;
    };

    /* Skip schema-qualified names - only collect default-schema types */
    let schema = target.schema();
    if !(schema.is_empty()
      || (schema.len() == 1 && schema[0].name().eq_ignore_ascii_case(DEFAULT_SCHEMA)))
    {
      continue;
    }
    let Some(name) = target.name() else {
      continue;
    };
    let name = name.name().to_string();
    let node = element_declaration.syntax();
    let symbol = Symbol::new(db, SymbolKind::UserTyp(file, node.clone(), name.clone()));

    /* Duplicate check: same name already declared in this file */
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

  /* Collect equality (alias) type declarations (`type Alias = Other`) */
  for equality_declaration in source_file.equality_declarations() {
    let Some(target) = equality_declaration.name() else {
      continue;
    };

    /* Skip schema-qualified names */
    let schema = target.schema();
    if !(schema.is_empty()
      || (schema.len() == 1 && schema[0].name().eq_ignore_ascii_case(DEFAULT_SCHEMA)))
    {
      continue;
    }

    let Some(name) = target.name() else {
      continue;
    };
    let name = name.name().to_string();
    let node = equality_declaration.syntax();
    let symbol = Symbol::new(db, SymbolKind::UserTyp(file, node.clone(), name.clone()));

    /* Duplicate check: conflicts with a block type or another alias of the same name */
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

  /* Collect symbols from use/reuse declarations */
  for use_declaration in source_file.use_declarations() {
    if reuses_only && !use_declaration.is_reuse() {
      continue;
    }
    collect_symbols_from_use_declaration(db, file, &use_declaration, members, top_schemas);
  }
}

/// Collects imported symbols from a single use declaration into `members`
fn collect_symbols_from_use_declaration<'db>(
  db: &'db dyn Database,
  file: File,
  use_declaration: &UseDeclaration,
  members: &mut StaticScopeMembers<'db>,
  top_schemas: &mut HashSet<String>,
) {
  let Some(specifier_path) = use_declaration.module_path() else {
    return;
  };
  let Some(module) = file.uses(db).get(&specifier_path) else {
    return;
  };

  /* Resolve module path to a symbol set */
  let target_symbols = match module {
    Either::Left(VirtualModuleKind::Metatype) => pretype_get_metatype_members(db),
    Either::Right(target_file) => {
      let symbols = pretype_get_file_symbols(db, *target_file, true).clone();
      /* Pull schema names from the target into the current file's top-schema set */
      for (_, symbol) in symbols.iter() {
        if let SymbolKind::UserSchema(..) = symbol.kind(db) {
          top_schemas.insert(symbol.name(db).to_string());
        }
      }
      symbols
    }
  };

  /* Wildcard: `use * from '...'` - import all non-schema symbols */
  if use_declaration.is_wildcard_use() {
    for (_, original_symbol) in target_symbols.iter() {
      if matches!(original_symbol.kind(db), SymbolKind::UserSchema(..)) {
        continue;
      }
      let visible_name = original_symbol.name(db).to_string();
      let node = use_declaration.syntax();

      if !check_import_collision(
        db,
        members,
        &visible_name,
        *original_symbol,
        node.start_offset(),
        node.end_offset(),
      ) {
        continue;
      }

      let use_symbol = Symbol::new(
        db,
        SymbolKind::Use(
          *original_symbol,
          file,
          use_declaration.syntax().clone(),
          visible_name,
        ),
      );
      members.insert(db, use_symbol);

      expand_namespace_if_needed(db, *original_symbol, members);
    }
    return;
  }

  /* Specifier list: `use { kind Name, kind Name as Alias } from '...'` */
  let Some(specifier_list) = use_declaration.specifier_list() else {
    return;
  };

  for specifier in specifier_list.specifiers() {
    let Some(specifier_name_fragment) = specifier.name().and_then(|node| node.name()) else {
      continue;
    };
    let specifier_name = specifier_name_fragment.name().to_string();
    let alias = specifier.alias();

    /* Visible name is the alias if present, otherwise the original name */
    let visible_name = alias
      .as_ref()
      .map(|alias_node| alias_node.text().trim().to_string())
      .unwrap_or_else(|| specifier_name.clone());
    let kind_fragment = specifier.kind();

    /* Find all symbols matching the name and optional kind filter */
    let matching_symbols: Vec<Symbol> = target_symbols
      .lookup_by_name(&specifier_name)
      .filter(|(_, symbol)| {
        kind_fragment
          .as_ref()
          .is_none_or(|kind| does_symbol_match_kind(db, file, kind, **symbol))
      })
      .map(|(_, symbol)| *symbol)
      .collect();

    if matching_symbols.is_empty() {
      /* Only emit UnresolvedImport for keyword kinds or bare names */
      let is_keyword_kind = kind_fragment.as_ref().is_some_and(is_keyword_kind);
      if is_keyword_kind || kind_fragment.is_none() {
        Diagnostics::accumulate(
          Diagnostics(
            DiagnosticKind::UnresolvedImport {
              name: specifier_name,
              module_path: specifier_path.clone(),
            }
            .into_error(
              specifier.syntax().start_offset(),
              specifier.syntax().end_offset(),
            ),
          ),
          db,
        );
      }
      continue;
    }

    /* Import each matched symbol as a Use symbol under the visible name */
    for original_symbol in matching_symbols {
      if matches!(original_symbol.kind(db), SymbolKind::UserSchema(..)) {
        continue;
      }
      let node = specifier.syntax();

      if !check_import_collision(
        db,
        members,
        &visible_name,
        original_symbol,
        node.start_offset(),
        node.end_offset(),
      ) {
        continue;
      }

      let use_symbol = Symbol::new(
        db,
        SymbolKind::Use(
          original_symbol,
          file,
          specifier.syntax().clone(),
          visible_name.clone(),
        ),
      );
      members.insert(db, use_symbol);

      expand_namespace_if_needed(db, original_symbol, members);
    }
  }
}

/// Expands `[on use: expand]` symbols by injecting their namespace members into `members`
fn expand_namespace_if_needed<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
  members: &mut StaticScopeMembers<'db>,
) {
  let Some(Obj::Typ(Typ::Element(element_typ))) = *pretype_evaluate_typ_symbol(db, symbol) else {
    return;
  };
  if element_typ.typ_settings(db).on_use != OnUsePolicy::ExpandReferences {
    return;
  }
  for (name, _) in element_typ.get_namespace_members(db) {
    let namespace_symbol = Symbol::new(db, SymbolKind::NamespaceMember(symbol, name.clone()));
    members.insert(db, namespace_symbol);
  }
}
