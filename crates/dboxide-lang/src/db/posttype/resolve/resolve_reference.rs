use salsa::Database;

use super::can_namespace_reach::symbol_namespace_reaches;
use crate::ast::RedNode;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{ElementTyp, File, Symbol, Typ};

use super::resolve_element_typ::resolve_element_typ;
use crate::db::prepost::get_symbol_namespace::get_symbol_namespace_posttype;
use crate::db::prepost::{
  resolve_alias_posttype, resolve_element_posttype, resolve_name_posttype, resolve_schema_posttype,
};

/// Resolves a reference expression to a symbol
/// When `target_typ` is given, restricts to elements whose type matches
/// When `target_typ` is absent, resolves any symbol by name
pub fn resolve_reference<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
  target_typ: Option<LazyTyp<'db>>,
) -> Option<Symbol<'db>> {
  let symbol = if target_typ.is_some() {
    (*resolve_element_posttype(db, file, node.clone(), name.clone()))
      .or_else(|| *resolve_alias_posttype(db, file, node, name))?
  } else {
    resolve_name_posttype(db, file, node.clone(), name.clone())
      .or_else(|| *resolve_element_posttype(db, file, node.clone(), name.clone()))
      .or_else(|| *resolve_alias_posttype(db, file, node, name))?
  };

  let Some(target_typ) = target_typ.and_then(|t| t.resolve(db)) else {
    return Some(symbol);
  };

  match symbol.original_symbol(db).kind(db) {
    SymbolKind::UserElement(file, node) => {
      let element_typ = resolve_element_typ(db, *file, node)?;
      if Typ::Element(element_typ) == target_typ {
        Some(symbol)
      } else {
        None
      }
    }
    _ => Some(symbol),
  }
}

/// Resolves a qualified reference (dot-separated path) to a symbol
/// When `target_typ` is given, guides traversal toward elements of that type
pub fn resolve_qualified_reference<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  parts: &[String],
  target_typ: Option<LazyTyp<'db>>,
) -> Option<Symbol<'db>> {
  if parts.is_empty() {
    return None;
  }

  let target_element_typ: Option<ElementTyp<'db>> = target_typ
    .and_then(|t| t.resolve(db))
    .and_then(|t| match t {
      Typ::Element(e) => Some(e),
      _ => None,
    });

  let mut current = resolve_name_posttype(db, file, node.clone(), parts[0].clone())
    .or_else(|| *resolve_schema_posttype(db, file, node.clone(), parts[0].clone()))
    .or_else(|| *resolve_element_posttype(db, file, node.clone(), parts[0].clone()))
    .or_else(|| *resolve_alias_posttype(db, file, node, parts[0].clone()))?;
  if let Some(target) = target_element_typ
    && !symbol_namespace_reaches(db, current, target)
  {
    return None;
  }

  let last_index = parts.len() - 1;
  for (index, part) in parts[1..].iter().enumerate() {
    let namespace = get_symbol_namespace_posttype(db, current);
    current = match (target_element_typ, index + 1 == last_index) {
      (Some(target), false) => *namespace
        .lookup_by_name(part)
        .find(|(_, s)| symbol_namespace_reaches(db, **s, target))
        .map(|(_, s)| s)?,
      _ => *namespace.lookup_by_name(part).next()?.1,
    };
  }

  Some(current)
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use itertools::Either;

  use crate::db::DboxideDatabase;
  use crate::db::posttype::resolve::resolve_reference::{
    resolve_qualified_reference, resolve_reference,
  };
  use crate::db::test_utils::*;
  use crate::db::types::interned::symbol::{SymbolKind, VirtualModuleKind};
  use crate::parse_file::parse_file;

  fn metatype() -> HashMap<String, Either<VirtualModuleKind, crate::db::types::File>> {
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )])
  }

  #[test]
  fn qualified_reference_resolves_schema_traversal() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

auth.type Foo {}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let resolved =
      resolve_qualified_reference(&db, file, root, &["auth".into(), "Foo".into()], None);
    assert!(resolved.is_some(), "expected auth.Foo to resolve");
  }

  #[test]
  fn resolve_reference_delegates_to_resolve_name() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Foo {}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let resolved = resolve_reference(&db, file, root, "Foo".to_string(), None);
    assert!(resolved.is_some());
  }

  #[test]
  fn nested_element_namespace_accessible_via_dot() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Column [field] {
  name declaration[Column] [label]
}

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  type Column [field] {
    col_name declaration[Column] [label]
  }
}

Table users {
  Column id
}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let resolved =
      resolve_qualified_reference(&db, file, root, &["users".into(), "id".into()], None);
    assert!(resolved.is_some(), "expected users.id to resolve");
  }

  #[test]
  fn reference_resolves_element_alias() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  alias qualified_declaration[Table] [label alias]
}

Table users as u {}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let resolved = resolve_reference(&db, file, root, "u".to_string(), None);
    assert!(resolved.is_some(), "expected alias u to resolve");
    assert!(
      matches!(resolved.unwrap().kind(&db), SymbolKind::ElementAlias(..)),
      "expected ElementAlias"
    );
  }

  #[test]
  fn deeply_nested_schema_traversal_resolves() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

a.b.type Nested {}
"#,
      metatype(),
    );

    let root = parse_file(&db, file).ast(&db).value(&db).node.clone();
    let resolved = resolve_qualified_reference(
      &db,
      file,
      root,
      &["a".into(), "b".into(), "Nested".into()],
      None,
    );
    assert!(resolved.is_some(), "expected a.b.Nested to resolve");
  }
}
