mod constraints;
mod pretype_evaluate_typ_alias;
mod pretype_evaluate_typ_element;
mod types;

use salsa::{Database, tracked};

use crate::ast::{AstNode, BlockElementDeclaration, EqualityDeclaration, RedNode};
use crate::db::types::interned::symbol::{SymbolKind, VirtualTypKind};
use crate::db::types::{
  AnyTyp, BoolTyp, ColorTyp, DeclarationTyp, File, FloatTyp, IntegerTyp, ListTyp, MapTyp, NullTyp,
  OStringTyp, Obj, QualifiedDeclarationTyp, QualifiedReferenceTyp, ReferenceTyp, StringTyp, Symbol,
  Typ,
};

use pretype_evaluate_typ_alias::pretype_evaluate_typ_alias_node;
use pretype_evaluate_typ_element::pretype_evaluate_typ_element_node;

/// Evaluate virtual / metatype built-in types
pub fn pretype_evaluate_virtual_typ_kind<'db>(
  db: &'db dyn Database,
  kind: &VirtualTypKind,
) -> Typ<'db> {
  match kind {
    VirtualTypKind::MetatypeString => StringTyp::new(db).into(),
    VirtualTypKind::MetatypeInteger => IntegerTyp::new(db).into(),
    VirtualTypKind::MetatypeFloat => FloatTyp::new(db).into(),
    VirtualTypKind::MetatypeBool => BoolTyp::new(db).into(),
    VirtualTypKind::MetatypeNull => NullTyp::new(db).into(),
    VirtualTypKind::MetatypeColor => ColorTyp::new(db).into(),
    VirtualTypKind::MetatypeAny => AnyTyp::new(db).into(),
    VirtualTypKind::MetatypeOString => OStringTyp::new(db).into(),
    VirtualTypKind::MetatypeDeclaration => DeclarationTyp::new(db, None).into(),
    VirtualTypKind::MetatypeQualifiedDeclaration => QualifiedDeclarationTyp::new(db, None).into(),
    VirtualTypKind::MetatypeReference => ReferenceTyp::new(db, None).into(),
    VirtualTypKind::MetatypeQualifiedReference => QualifiedReferenceTyp::new(db, None).into(),
    VirtualTypKind::MetatypeMap => MapTyp::new(db, None, None).into(),
    VirtualTypKind::MetatypeList => ListTyp::new(db, None).into(),
  }
}

#[tracked]
pub fn pretype_evaluate_typ_symbol<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> Option<Obj<'db>> {
  match symbol.kind(db) {
    SymbolKind::VirtualTyp(_, kind) => Some(Obj::Typ(pretype_evaluate_virtual_typ_kind(db, kind))),
    SymbolKind::UserTyp(file, node) => {
      pretype_evaluate_user_typ_kind(db, *file, symbol.name(db), node).map(Obj::Typ)
    }
    SymbolKind::Use(target, _, _) => *pretype_evaluate_typ_symbol(db, *target),
    _ => None,
  }
}

fn pretype_evaluate_user_typ_kind<'db>(
  db: &'db dyn Database,
  file: File,
  name: &str,
  node: &RedNode,
) -> Option<Typ<'db>> {
  if let Some(equality_declaration) = EqualityDeclaration::cast(node.clone()) {
    return pretype_evaluate_typ_alias_node(db, file, &equality_declaration);
  }

  if let Some(block_declaration) = BlockElementDeclaration::cast(node.clone()) {
    return Some(pretype_evaluate_typ_element_node(
      db,
      file,
      name,
      &block_declaration,
    ));
  }

  None
}
