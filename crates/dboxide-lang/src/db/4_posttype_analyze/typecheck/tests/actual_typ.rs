use super::*;
use crate::ast::AstNode;

#[test]
fn typ_ident_returns_element_typ() {
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
  let root = parse_root(&db, file);
  let source_file = SourceFile::cast(root).unwrap();
  let element = source_file
    .element_declarations()
    .find(|element| {
      element
        .target_name()
        .is_some_and(|name| name.text().trim() == "users")
    })
    .unwrap();
  let table_node = element.typ().unwrap().name().unwrap().node();
  assert!(matches!(
    *get_node_actual_typ(&db, file, table_node),
    Some(Typ::Element(_))
  ));
}

#[test]
fn element_ident_returns_element_typ() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  ref reference[Table] [attribute]
}

Table users {}
Table orders {
  ref: users
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let users_node = find_ident_last(&root, "users").unwrap();
  assert!(matches!(
    *get_node_actual_typ(&db, file, users_node),
    Some(Typ::Element(_))
  ));
}

#[test]
fn integer_literal_is_integer() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  constraints {
    check: () => 42 == 42
  }
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let num = find_first_by_kind(&root, SyntaxKind::NumberExpr).unwrap();
  assert!(matches!(
    *get_node_actual_typ(&db, file, num),
    Some(Typ::Integer(_))
  ));
}

#[test]
fn string_literal_is_string() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  note string [setting]
}

Table users [note: "hello"] {}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let str_node = find_first_by_kind(&root, SyntaxKind::DqStringExpr).unwrap();
  assert!(matches!(
    *get_node_actual_typ(&db, file, str_node),
    Some(Typ::String(_))
  ));
}

#[test]
fn comparison_is_bool() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  constraints {
    check: () => 1 == 2
  }
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let eq = find_infix(&root, "==").unwrap();
  assert!(matches!(
    *get_node_actual_typ(&db, file, eq),
    Some(Typ::Bool(_))
  ));
}

#[test]
fn logical_and_is_bool() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  constraints {
    check: () => true && false
  }
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let and = find_infix(&root, "&&").unwrap();
  assert!(matches!(
    *get_node_actual_typ(&db, file, and),
    Some(Typ::Bool(_))
  ));
}

#[test]
fn list_literal_is_list() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  constraints {
    check: () => [1, 2, 3] == [1, 2, 3]
  }
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let list = find_first_by_kind(&root, SyntaxKind::ListExpr).unwrap();
  assert!(matches!(
    *get_node_actual_typ(&db, file, list),
    Some(Typ::List(_))
  ));
}

#[test]
fn dot_chain_resolves_to_namespace_member_type() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  ref reference[Table] [attribute]
}

Table users {}
Table orders {
  ref: users
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let users_node = find_ident_last(&root, "users").unwrap();
  assert!(matches!(
    *get_node_actual_typ(&db, file, users_node),
    Some(Typ::Element(_))
  ));
}

#[test]
fn sum_typ_deduplicates_identical_types() {
  use crate::db::types::{IntegerTyp, SumTyp};
  use salsa::tracked;

  #[tracked]
  fn run(db: &dyn salsa::Database) {
    let int = Typ::Integer(IntegerTyp::new(db));
    let result = SumTyp::create(db, vec![int, int]);
    assert!(matches!(result, Typ::Integer(_)));
  }
  run(&DboxideDatabase::default());
}

#[test]
fn sum_typ_preserves_distinct_types_as_union() {
  use crate::db::types::{IntegerTyp, StringTyp, SumTyp};
  use salsa::tracked;

  #[tracked]
  fn run(db: &dyn salsa::Database) {
    let int_typ = Typ::Integer(IntegerTyp::new(db));
    let str_typ = Typ::String(StringTyp::new(db));
    let result = SumTyp::create(db, vec![int_typ, str_typ]);
    assert!(matches!(result, Typ::Sum(_)));
    if let Typ::Sum(sum) = result {
      assert_eq!(sum.members(db).len(), 2);
    }
  }
  run(&DboxideDatabase::default());
}

#[test]
fn sum_typ_flattens_nested_sums() {
  use crate::db::types::{BoolTyp, IntegerTyp, StringTyp, SumTyp};
  use salsa::tracked;

  #[tracked]
  fn run(db: &dyn salsa::Database) {
    let int_typ = Typ::Integer(IntegerTyp::new(db));
    let str_typ = Typ::String(StringTyp::new(db));
    let bool_typ = Typ::Bool(BoolTyp::new(db));
    let inner_sum = SumTyp::create(db, vec![int_typ, str_typ]);
    let outer = SumTyp::create(db, vec![inner_sum, bool_typ]);
    assert!(matches!(outer, Typ::Sum(_)));
    if let Typ::Sum(sum) = outer {
      assert_eq!(sum.members(db).len(), 3);
    }
  }
  run(&DboxideDatabase::default());
}

#[test]
fn sum_without_null_collapses_to_single_type() {
  use crate::db::types::{NullTyp, StringTyp, SumTyp};
  use salsa::tracked;

  #[tracked]
  fn run(db: &dyn salsa::Database) {
    let str_typ = Typ::String(StringTyp::new(db));
    let null_typ = Typ::Null(NullTyp::new(db));
    let nullable = SumTyp::create(db, vec![str_typ, null_typ]);
    assert!(matches!(nullable, Typ::Sum(_)));
    let non_null = SumTyp::create(db, vec![str_typ]);
    assert!(matches!(non_null, Typ::String(_)));
  }
  run(&DboxideDatabase::default());
}
