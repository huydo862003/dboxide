use super::*;
use crate::ast::{AstNode, InfixExpr, PrefixExpr, TernaryExpr};

#[test]
fn type_annotation_position_returns_none() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let ident = find_ident_last(&root, "qualified_declaration").unwrap();
  assert!(get_node_expected_typ(&db, file, ident).is_none());
}

#[test]
fn and_operand_expects_bool() {
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
  let and_node = find_infix(&root, "&&").unwrap();
  let lhs = InfixExpr::cast(and_node).unwrap().left().unwrap();
  assert!(matches!(
    *get_node_expected_typ(&db, file, lhs.syntax().clone()),
    Some(Typ::Bool(_))
  ));
}

#[test]
fn or_operand_expects_bool() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  constraints {
    check: () => true || false
  }
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let or_node = find_infix(&root, "||").unwrap();
  let rhs = InfixExpr::cast(or_node).unwrap().right().unwrap();
  assert!(matches!(
    *get_node_expected_typ(&db, file, rhs.syntax().clone()),
    Some(Typ::Bool(_))
  ));
}

#[test]
fn not_operand_expects_bool() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  constraints {
    check: () => !true
  }
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let prefix = find_first_by_kind(&root, SyntaxKind::PrefixExpr).unwrap();
  let operand = PrefixExpr::cast(prefix).unwrap().operand().unwrap();
  assert!(matches!(
    *get_node_expected_typ(&db, file, operand.syntax().clone()),
    Some(Typ::Bool(_))
  ));
}

#[test]
fn ternary_condition_expects_bool() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  constraints {
    check: () => true ? 1 : 0
  }
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let ternary_node = find_first_by_kind(&root, SyntaxKind::TernaryExpr).unwrap();
  let ternary = TernaryExpr::cast(ternary_node).unwrap();
  let cond = ternary.condition().unwrap();
  assert!(matches!(
    *get_node_expected_typ(&db, file, cond.syntax().clone()),
    Some(Typ::Bool(_))
  ));
}

#[test]
fn coalesce_lhs_expects_nullable() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  constraints {
    check: () => (true ?? false) && true
  }
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let coalesce = find_infix(&root, "??").unwrap();
  let infix = InfixExpr::cast(coalesce).unwrap();
  let lhs = infix.left().unwrap();
  // Inside && context, parent expected is Bool, so LHS expected is Bool | null
  assert!(matches!(
    *get_node_expected_typ(&db, file, lhs.syntax().clone()),
    Some(Typ::Sum(_)) | Some(Typ::Bool(_))
  ));
}

#[test]
fn attribute_value_has_expected_type() {
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

Table users {
  ref: users
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let users_node = find_ident_last(&root, "users").unwrap();
  assert!(get_node_expected_typ(&db, file, users_node).is_some());
}

#[test]
fn setting_value_has_expected_type() {
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
    *get_node_expected_typ(&db, file, str_node),
    Some(Typ::String(_))
  ));
}

#[test]
fn index_subscript_expects_integer_for_list() {
  let db = DboxideDatabase::default();
  let file = create_file(
    &db,
    "test.dbml",
    r#"
use * from 'metatype'

type Table [element] {
  name qualified_declaration[Table] [label, unique]
  constraints {
    check: () => [1, 2, 3][0] == 1
  }
}
"#,
    metatype(),
  );
  let root = parse_root(&db, file);
  let index_expr = find_first_by_kind(&root, SyntaxKind::IndexExpr).unwrap();
  let subscript = index_expr
    .children()
    .filter(|node| !node.kind().is_trivia())
    .nth(1)
    .unwrap();
  let num = find_first_by_kind(&subscript, SyntaxKind::NumberExpr).unwrap_or(subscript);
  assert!(matches!(
    *get_node_expected_typ(&db, file, num),
    Some(Typ::Integer(_))
  ));
}
