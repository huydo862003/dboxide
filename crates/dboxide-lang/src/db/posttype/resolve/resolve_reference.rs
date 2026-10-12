use salsa::Database;

use super::can_namespace_reach::symbol_namespace_reaches;
use crate::ast::RedNode;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{ElementTyp, File, Symbol, Typ};

use super::resolve_element_typ::resolve_element_typ;
use crate::db::prepost::get_symbol_namespace::get_symbol_namespace_posttype;
use crate::db::prepost::resolve_name_posttype as resolve_name;

// Resolve a reference expression to a symbol
// For ref[T]: search scope for elements whose type is T
// For qualified_reference[T]: support schema.table.column traversal
pub fn resolve_reference<'db>(
  db: &'db dyn Database,
  file: File,
  node: RedNode,
  name: String,
  target_typ: Option<LazyTyp<'db>>,
) -> Option<Symbol<'db>> {
  let symbol = (*resolve_name(db, file, node, name))?;

  let Some(target_typ) = target_typ else {
    return Some(symbol);
  };
  let Some(target_typ) = target_typ.resolve(db) else {
    return Some(symbol);
  };

  let kind = match symbol.kind(db) {
    SymbolKind::ElementAlias(primary) => primary.kind(db).clone(),
    other => other.clone(),
  };
  match kind {
    SymbolKind::UserElement(file, ref node) => {
      let element_typ = resolve_element_typ(db, file, node)?;
      if Typ::Element(element_typ) == target_typ {
        Some(symbol)
      } else {
        None
      }
    }
    _ => Some(symbol),
  }
}

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

  let mut current = {
    let candidates = resolve_name(db, file, node.clone(), parts[0].clone());
    let symbol = (*candidates)?;
    // If there are multiple candidates (name collision), pick one that can reach target
    if let Some(target) = target_element_typ
      && !symbol_namespace_reaches(db, symbol, target)
    {
      return None;
    }
    symbol
  };

  let last_index = parts.len() - 1;
  for (index, part) in parts[1..].iter().enumerate() {
    let namespace = get_symbol_namespace_posttype(db, current);
    current = if let Some(target) = target_element_typ {
      if index + 1 == last_index {
        // Final step: caller handles type matching; no reachability filter needed
        *namespace.lookup_by_name(part).next()?.1
      } else {
        // Intermediate step: pick candidate whose namespace can reach target
        *namespace
          .lookup_by_name(part)
          .find(|(_, s)| symbol_namespace_reaches(db, **s, target))
          .map(|(_, s)| s)?
      }
    } else {
      *namespace.lookup_by_name(part).next()?.1
    };
  }

  Some(current)
}
