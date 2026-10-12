use salsa::{Accumulator, Database};

use crate::ast::{AstNode, ElementDeclaration, RedNode};
use crate::db::types::interned::symbol::SymbolKind;
use crate::db::types::{Diagnostics, File, Symbol};
use crate::diagnostics::{Diagnostic, DiagnosticKind, Severity};

use crate::db::posttype::resolve::resolve_element_typ::resolve_element_typ;
use crate::db::types::Typ;

// Given an element instance node, extract declared symbols from label fields
pub fn declare_element_symbols<'db>(
  db: &'db dyn Database,
  file: File,
  element_node: &RedNode,
) -> Vec<Symbol<'db>> {
  let mut symbols = Vec::new();

  let Some(element) = ElementDeclaration::cast(element_node.clone()) else {
    return symbols;
  };

  let Some(element_typ) = resolve_element_typ(db, file, element_node) else {
    return symbols;
  };

  let file_path = file.filepath(db).display().to_string();
  let labels = element_typ.label_definitions(db);

  let has_primary_label = labels.iter().any(|label| !label.is_alias);
  if let Some(name_node) = element.target_name()
    && !name_node.text().trim().is_empty()
  {
    let name = name_node.text().trim().to_string();
    if has_primary_label {
      let symbol = Symbol::new(
        db,
        SymbolKind::UserElement(file, element_node.clone()),
        name.clone(),
        format!("{file_path}::{name}"),
      );
      symbols.push(symbol);
    } else {
      let start = name_node.syntax().offset();
      let end = start + name_node.syntax().text_len();
      let typ_name = element_typ.name(db).clone();
      Diagnostics(Diagnostic {
        kind: DiagnosticKind::NameNotSupportedByTyp { typ_name },
        severity: Severity::Error,
        start_offset: start,
        end_offset: end,
      })
      .accumulate(db);
    }
  }

  let has_alias_label = labels.iter().any(|label| label.is_alias);
  if let Some(alias_node) = element.alias() {
    let start = alias_node.syntax().offset();
    let end = start + alias_node.syntax().text_len();
    if !has_alias_label {
      let typ_name = element_typ.name(db).clone();
      Diagnostics(Diagnostic {
        kind: DiagnosticKind::AliasNotSupportedByTyp { typ_name },
        severity: Severity::Error,
        start_offset: start,
        end_offset: end,
      })
      .accumulate(db);
    } else if let Some(alias_name_node) = alias_node.name() {
      let alias_name = alias_name_node.text().trim().to_string();
      if !alias_name.is_empty() {
        if let Some(&primary_symbol) = symbols.first() {
          let symbol = Symbol::new(
            db,
            SymbolKind::ElementAlias(primary_symbol),
            alias_name.clone(),
            format!("{file_path}::@alias::{alias_name}"),
          );
          symbols.push(symbol);
        } else {
          Diagnostics(Diagnostic {
            kind: DiagnosticKind::AliasMissingPrimaryName,
            severity: Severity::Error,
            start_offset: start,
            end_offset: end,
          })
          .accumulate(db);
        }
      }
    }
  }

  // Unlabeled elements with reference-typed fields are implicitly in scope —
  // they represent relationships whose endpoints (referenced elements) are imported into scope
  if symbols.is_empty() {
    let has_reference_field = element_typ.field_definitions(db).iter().any(|f| {
      matches!(
        f.typ.resolve(db),
        Some(Typ::Reference(_) | Typ::QualifiedReference(_))
      )
    });
    if has_reference_field {
      let start = element_node.offset();
      let symbol = Symbol::new(
        db,
        SymbolKind::UserElement(file, element_node.clone()),
        format!("@anon::{start}"),
        format!("{file_path}::@anon::{start}"),
      );
      symbols.push(symbol);
    }
  }

  symbols
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use itertools::Either;

  use crate::db::DboxideDatabase;
  use crate::db::prepost::get_scope_members::get_scope_members_posttype;
  use crate::db::test_utils::*;
  use crate::db::types::interned::symbol::{SymbolKind, VirtualModuleKind};
  use crate::db::types::{Diagnostics, StaticScope, StaticScopeKind};
  use crate::diagnostics::DiagnosticKind;

  fn get_file_scope_members(
    db: &DboxideDatabase,
    file: crate::db::types::File,
  ) -> crate::db::types::interned::symbol::StaticScopeMembers<'_> {
    let scope = StaticScope::new(db, StaticScopeKind::UserModule(file));
    get_scope_members_posttype(db, scope).clone()
  }

  fn metatype() -> HashMap<String, Either<VirtualModuleKind, crate::db::types::File>> {
    HashMap::from([(
      "metatype".to_string(),
      Either::Left(VirtualModuleKind::Metatype),
    )])
  }

  #[test]
  fn table_declares_symbol_in_scope() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users {}
"#,
      metatype(),
    );

    let members = get_file_scope_members(&db, file);
    assert_contains_symbol(&members, "users");
    assert!(matches!(
      get_first_symbol(&members, "users").kind(&db),
      SymbolKind::UserElement(..)
    ));
    assert_contains_symbol(&members, "Table");
  }

  #[test]
  fn table_alias_declares_element_alias() {
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

    let members = get_file_scope_members(&db, file);
    assert_contains_symbol(&members, "users");
    assert_contains_symbol(&members, "u");
    assert!(matches!(
      get_first_symbol(&members, "users").kind(&db),
      SymbolKind::UserElement(..)
    ));
    assert!(matches!(
      get_first_symbol(&members, "u").kind(&db),
      SymbolKind::ElementAlias(..)
    ));
  }

  #[test]
  fn element_without_label_declares_no_symbol() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Note [element] {
  content string [arg: 0]
}

Note { 'hello' }
"#,
      metatype(),
    );

    let members = get_file_scope_members(&db, file);
    assert_contains_symbol(&members, "Note");
  }

  #[test]
  fn name_on_unlabeled_type_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Note [element] {
  content string [arg: 0]
}

Note myname { 'hello' }
"#,
      metatype(),
    );

    let scope = StaticScope::new(&db, StaticScopeKind::UserModule(file));
    let diagnostics = get_scope_members_posttype::accumulated::<Diagnostics>(&db, scope);
    assert!(
      diagnostics
        .iter()
        .any(|d| matches!(d.0.kind, DiagnosticKind::NameNotSupportedByTyp { .. })),
      "expected NameNotSupportedByType diagnostic"
    );
  }

  #[test]
  fn alias_without_primary_name_emits_diagnostic() {
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

Table as u {}
"#,
      metatype(),
    );

    let scope = StaticScope::new(&db, StaticScopeKind::UserModule(file));
    let diagnostics = get_scope_members_posttype::accumulated::<Diagnostics>(&db, scope);
    assert!(
      diagnostics
        .iter()
        .any(|d| matches!(d.0.kind, DiagnosticKind::AliasMissingPrimaryName)),
      "expected AliasMissingPrimaryName diagnostic"
    );
  }

  #[test]
  fn alias_on_type_without_alias_label_emits_diagnostic() {
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

Table users as u {}
"#,
      metatype(),
    );

    let scope = StaticScope::new(&db, StaticScopeKind::UserModule(file));
    let diagnostics = get_scope_members_posttype::accumulated::<Diagnostics>(&db, scope);
    assert!(
      diagnostics
        .iter()
        .any(|d| matches!(d.0.kind, DiagnosticKind::AliasNotSupportedByTyp { .. })),
      "expected AliasNotSupportedByType diagnostic"
    );
  }

  #[test]
  fn unlabeled_reference_element_gets_anonymous_symbol() {
    // Elements whose type has reference-typed fields but no label field are implicitly
    // in scope as anonymous symbols (name starts with "@anon::")
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}

type Ref [element] {
  from reference[Table] [arg: 0]
  to reference[Table] [arg: 1]
}

Table users {}
Table orders {}
Ref {}
"#,
      metatype(),
    );

    let members = get_file_scope_members(&db, file);
    // labeled elements are in scope normally
    assert_contains_symbol(&members, "users");
    assert_contains_symbol(&members, "orders");
    // unlabeled Ref element is in scope as an anonymous symbol
    assert!(
      members.iter().any(|(key, _)| key.0.starts_with("@anon::")),
      "expected anonymous symbol for the unlabeled Ref element"
    );
  }

  #[test]
  fn unlabeled_non_reference_element_gets_no_anonymous_symbol() {
    // Elements without reference-typed fields do not get an anonymous symbol
    let db = DboxideDatabase::default();
    let file = create_file(
      &db,
      "test.dbml",
      r#"
use * from 'metatype'

type Note [element] {
  content string [arg: 0]
}

Note { 'hello' }
"#,
      metatype(),
    );

    let members = get_file_scope_members(&db, file);
    assert!(
      !members.iter().any(|(key, _)| key.0.starts_with("@anon::")),
      "non-reference unlabeled element should not produce an anonymous symbol"
    );
  }
}
