mod actual_typ;
mod expected_typ;

use std::collections::HashMap;

use itertools::Either;

use crate::ast::{AstNode, IdentExpr, InfixExpr, RedNode, SourceFile, SyntaxKind};
use crate::db::DboxideDatabase;
use crate::db::posttype_analyze::typecheck::actual_typ::get_node_actual_typ;
use crate::db::posttype_analyze::typecheck::get_node_expected_typ;
use crate::db::test_utils::*;
use crate::db::types::interned::symbol::VirtualModuleKind;
use crate::db::types::{File, Typ};
use crate::parse_file::parse_file;

fn find_ident_last(node: &RedNode, target: &str) -> Option<RedNode> {
  let mut result = None;
  if let Some(ident) = IdentExpr::cast(node.clone())
    && ident.name() == target
  {
    result = Some(node.clone());
  }
  for child in node.children() {
    if let Some(found) = find_ident_last(&child, target) {
      result = Some(found);
    }
  }
  result
}

fn find_first_by_kind(node: &RedNode, kind: SyntaxKind) -> Option<RedNode> {
  if node.kind() == kind {
    return Some(node.clone());
  }
  for child in node.children() {
    if let Some(found) = find_first_by_kind(&child, kind) {
      return Some(found);
    }
  }
  None
}

fn find_all_by_kind(node: &RedNode, kind: SyntaxKind) -> Vec<RedNode> {
  let mut results = Vec::new();
  if node.kind() == kind {
    results.push(node.clone());
  }
  for child in node.children() {
    results.extend(find_all_by_kind(&child, kind));
  }
  results
}

/// Find an InfixExpr with a specific operator
fn find_infix(node: &RedNode, op: &str) -> Option<RedNode> {
  find_all_by_kind(node, SyntaxKind::InfixExpr)
    .into_iter()
    .find(|node| {
      InfixExpr::cast(node.clone())
        .and_then(|infix| infix.op())
        .as_deref()
        == Some(op)
    })
}

fn metatype() -> HashMap<String, Either<VirtualModuleKind, File>> {
  HashMap::from([(
    "metatype".to_string(),
    Either::Left(VirtualModuleKind::Metatype),
  )])
}

fn parse_root(db: &DboxideDatabase, file: File) -> RedNode {
  parse_file(db, file).ast(db).value(db).node.clone()
}
