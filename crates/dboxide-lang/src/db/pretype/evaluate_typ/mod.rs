mod constraints;
mod typ_alias;
mod typ_element;
pub(crate) mod types;

#[cfg(test)]
mod tests;

use salsa::{Database, tracked};

use crate::ast::{AstNode, BlockElementDeclaration, EqualityDeclaration, RedNode};
use crate::db::types::interned::symbol::{SymbolKind, VirtualTypKind};
use crate::db::types::{
  AnyTyp, BoolTyp, ColorTyp, DeclarationTyp, File, FloatTyp, IntegerTyp, ListTyp, MapTyp, NullTyp,
  OStringTyp, Obj, QualifiedDeclarationTyp, QualifiedReferenceTyp, ReferenceTyp, StaticTyp,
  StringTyp, Symbol, Typ,
};

use typ_alias::evaluate_typ_alias;
use typ_element::evaluate_typ_element;

#[tracked]
pub fn evaluate_typ_symbol<'db>(db: &'db dyn Database, symbol: Symbol<'db>) -> Option<Obj<'db>> {
  match symbol.kind(db) {
    SymbolKind::VirtualTyp(_, kind) => Some(Obj::Typ(evaluate_virtual_typ(db, kind))),
    SymbolKind::UserTyp(file, node) => {
      pretype_evaluate_user_typ_kind(db, *file, symbol.name(db), node).map(Obj::Typ)
    }
    SymbolKind::Use(target, _, _) => *evaluate_typ_symbol(db, *target),
    SymbolKind::NamespaceMember(type_sym, member_name) => {
      let typ = (*evaluate_typ_symbol(db, *type_sym))?.as_typ()?;
      let lazy = typ.lookup_namespace_member(db, member_name)?;
      lazy.resolve(db).map(Obj::Typ)
    }
    _ => None,
  }
}

/// Evaluate virtual / metatype built-in types
fn evaluate_virtual_typ<'db>(db: &'db dyn Database, kind: &VirtualTypKind) -> Typ<'db> {
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

fn pretype_evaluate_user_typ_kind<'db>(
  db: &'db dyn Database,
  file: File,
  name: &str,
  node: &RedNode,
) -> Option<Typ<'db>> {
  if let Some(equality_declaration) = EqualityDeclaration::cast(node.clone()) {
    return evaluate_typ_alias(db, file, &equality_declaration);
  }

  if let Some(block_declaration) = BlockElementDeclaration::cast(node.clone()) {
    return Some(evaluate_typ_element(db, file, name, &block_declaration));
  }

  None
}
