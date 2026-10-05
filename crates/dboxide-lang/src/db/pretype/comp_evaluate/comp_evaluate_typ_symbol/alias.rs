use salsa::Database;

use crate::ast::EqualityDeclaration;
use crate::db::pretype::comp_evaluate::comp_evaluate_node;
use crate::db::types::interned::symbol::VirtualTypKind;
use crate::db::types::{
  AnyTyp, BoolTyp, ColorTyp, DeclarationTyp, File, FloatTyp, IntegerTyp, ListTyp, MapTyp, NullTyp,
  OStringTyp, QualifiedDeclarationTyp, QualifiedReferenceTyp, ReferenceTyp, StringTyp, Typ,
};

/// Evaluate virtual / metatype built-in types
pub fn evaluate_virtual_typ_kind<'db>(db: &'db dyn Database, kind: &VirtualTypKind) -> Typ<'db> {
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

/// Evaluate `type Name = expr` by resolving the RHS
pub fn evaluate_typ_alias_node<'db>(
  db: &'db dyn Database,
  file: File,
  equality_declaration: &EqualityDeclaration,
) -> Option<Typ<'db>> {
  let rhs = equality_declaration.rhs()?;
  comp_evaluate_node(db, file, &rhs)?.as_typ()
}
