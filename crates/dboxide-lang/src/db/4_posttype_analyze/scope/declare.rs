use std::collections::{BTreeMap, HashSet};

use salsa::{Accumulator, Database};

use crate::ElementTyp;
use crate::ast::{AstNode, ElementDeclaration, ElementFieldDeclaration, FuncDeclaration, RedNode};
use crate::db::pretype_analyze::get_declaration_typ;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{Diagnostics, File, Symbol};
use crate::diagnostics::{Diagnostic, DiagnosticKind, Severity};

use crate::db::types::Typ;

/// Declares symbols for an element instance: primary name, alias, or anonymous symbol for unlabeled elements
pub fn declare_element_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  element_node: &RedNode,
) -> Vec<Symbol<'db>> {
  let mut symbols = Vec::new();

  let Some(element) = ElementDeclaration::cast(element_node.clone()) else {
    return symbols;
  };

  let Some(Typ::Element(element_typ)) = *get_declaration_typ(db, file, element_node.clone()) else {
    return symbols;
  };
  let labels = element_typ.label_definitions(db);

  /* Primary name: `Type name { ... }` */
  let has_primary_label = labels.iter().any(|label| !label.is_alias);
  if let Some(name_node) = element.target_name()
    && !name_node.text().trim().is_empty()
  {
    let name = name_node.text().trim().to_string();
    if has_primary_label {
      /* Typ supports [label] - declare the element symbol */
      let symbol = Symbol::new(
        db,
        SymbolKind::UserElement(file, element_node.clone(), name.clone()),
      );
      symbols.push(symbol);
    } else {
      /* Typ has no [label] field - name is not allowed */
      Diagnostics(Diagnostic {
        kind: DiagnosticKind::NameNotSupportedByTyp {
          typ_name: element_typ.name(db).clone(),
        },
        severity: Severity::Error,
        start_offset: name_node.syntax().start_offset(),
        end_offset: name_node.syntax().end_offset(),
      })
      .accumulate(db);
    }
  }

  /* Alias: `Type name as alias { ... }` */
  let has_alias_label = labels.iter().any(|label| label.is_alias);
  if let Some(alias_node) = element.alias() {
    if !has_alias_label {
      /* Typ has no [label alias] field - alias is not allowed */
      Diagnostics(Diagnostic {
        kind: DiagnosticKind::AliasNotSupportedByTyp {
          typ_name: element_typ.name(db).clone(),
        },
        severity: Severity::Error,
        start_offset: alias_node.syntax().start_offset(),
        end_offset: alias_node.syntax().end_offset(),
      })
      .accumulate(db);
    } else if let Some(alias_name_node) = alias_node.name() {
      let alias_name = alias_name_node.text().trim().to_string();
      if !alias_name.is_empty() {
        if let Some(&primary_symbol) = symbols.first() {
          /* Alias links to the primary symbol */
          let symbol = Symbol::new(
            db,
            SymbolKind::ElementAlias(primary_symbol, alias_name.clone()),
          );
          symbols.push(symbol);
        } else {
          /* Alias without a primary name - not valid */
          Diagnostics(Diagnostic {
            kind: DiagnosticKind::AliasMissingPrimaryName,
            severity: Severity::Error,
            start_offset: alias_node.syntax().start_offset(),
            end_offset: alias_node.syntax().end_offset(),
          })
          .accumulate(db);
        }
      }
    }
  }

  /* Unlabeled elements with reference arg fields are still in scope (for wildcard imports of anonymous Refs) */
  if symbols.is_empty() {
    let has_reference_field = has_reference_arg_field(db, element_typ);
    if has_reference_field {
      let _start = element_node.start_offset();
      symbols.push(Symbol::new(
        db,
        SymbolKind::AnonymousElement(file, element_node.clone()),
      ));
    }
  }

  symbols
}

/// Returns true if the element type has any reference-typed arg fields (direct or via field line type)
fn has_reference_arg_field(db: &dyn Database, element_typ: ElementTyp) -> bool {
  let is_reference = |typ: Typ| {
    let inner = match typ {
      Typ::List(list) => list.element(db).and_then(|element| element.resolve(db)),
      other => Some(other),
    };
    matches!(inner, Some(Typ::Reference(_) | Typ::QualifiedReference(_)))
  };

  // Check direct top-level fields
  if element_typ
    .field_definitions(db)
    .iter()
    .any(|field| field.typ.resolve(db).is_some_and(&is_reference))
  {
    return true;
  }

  // Check arg fields of the field line type (e.g. RefBinary inside Ref)
  if let Some(field_typ) = element_typ.field_line_typ(db)
    && field_typ
      .typ_element_arg_fields(db)
      .values()
      .any(|arg| arg.typ.resolve(db).is_some_and(&is_reference))
  {
    return true;
  }

  false
}

/// Declares a symbol for a function or operator declaration
pub fn declare_func_symbol<'db>(
  db: &'db dyn Database,
  file: File,
  func_declaration: &FuncDeclaration,
) -> Option<Symbol<'db>> {
  let name_node = func_declaration.name()?;
  if name_node.is_operator_declaration() {
    let op_symbol = name_node.operator_symbol()?;
    Some(Symbol::new(
      db,
      SymbolKind::UserOperator(file, func_declaration.syntax().clone(), op_symbol),
    ))
  } else {
    let name = name_node.text().trim().to_string();
    Some(Symbol::new(
      db,
      SymbolKind::UserFunction(file, func_declaration.syntax().clone(), name),
    ))
  }
}

/// Declares symbols from a field line: one symbol per labeled arg field on the field type
pub fn declare_field_line_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  field: &ElementFieldDeclaration,
  field_nested_typs: &BTreeMap<String, LazyTyp<'db>>,
) -> Vec<Symbol<'db>> {
  let args: Vec<_> = field.args().collect();
  if args.is_empty() {
    return vec![];
  }

  let field_element_typ =
    field_nested_typs
      .values()
      .find_map(|lazy_typ| match lazy_typ.resolve(db) {
        Some(Typ::Element(element_typ)) => Some(element_typ),
        _ => None,
      });
  let Some(field_element_typ) = field_element_typ else {
    return vec![];
  };

  /* Find labeled arg fields and create a symbol for each */
  let label_field_names: HashSet<String> = field_element_typ
    .label_definitions(db)
    .into_iter()
    .filter(|label| !label.is_alias)
    .map(|label| label.field_name)
    .collect();

  /* Find labeled arg fields and create a symbol for each */
  let mut symbols = Vec::new();
  for (position, arg_definition) in field_element_typ.typ_element_arg_fields(db) {
    if !label_field_names.contains(&arg_definition.name) {
      continue;
    }
    let arg_index = *position;
    let Some(arg) = args.get(arg_index) else {
      continue;
    };
    let name = arg.text().trim().to_string();
    if name.is_empty() {
      continue;
    }
    symbols.push(Symbol::new(
      db,
      SymbolKind::UserElement(file, field.syntax().clone(), name.clone()),
    ));
  }
  symbols
}
