mod constraints;
pub(crate) mod evaluate_node;
mod typ_alias;
mod typ_element;
pub(crate) mod types;

#[cfg(test)]
mod tests;

use salsa::{Database, tracked};

use crate::ast::{AstNode, BlockElementDeclaration, EqualityDeclaration, RedNode};
use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::symbol::{
  SymbolKind, VirtualFunctionKind, VirtualOperatorKind, VirtualTypKind,
};
use crate::db::types::{
  BoolTyp, ColorTyp, DeclarationTyp, File, FloatTyp, FuncObj, FuncTyp, IntegerTyp, ListTyp, MapTyp,
  NullTyp, OStringTyp, Obj, ObjTyp, QualifiedDeclarationTyp, QualifiedReferenceTyp, ReferenceTyp,
  StaticTyp, StringTyp, Symbol, Typ,
};

use typ_alias::pretype_evaluate_typ_alias;
use typ_element::pretype_evaluate_typ_element;

/// Evaluates a symbol to its type-level object (Typ, FuncObj, or None for non-type symbols)
#[tracked]
pub fn pretype_evaluate_typ_symbol<'db>(
  db: &'db dyn Database,
  symbol: Symbol<'db>,
) -> Option<Obj<'db>> {
  match symbol.kind(db) {
    SymbolKind::VirtualTyp(_, kind) => Some(Obj::Typ(evaluate_virtual_typ(db, kind))),
    SymbolKind::VirtualFunction(_, kind) => Some(Obj::Func(make_virtual_func_obj(db, kind))),
    SymbolKind::VirtualOperator(kind) => Some(Obj::Func(make_virtual_operator_obj(db, kind))),
    SymbolKind::UserTyp(file, node, _) => {
      pretype_evaluate_user_typ_kind(db, *file, &symbol.name(db), node).map(Obj::Typ)
    }
    SymbolKind::Use(target, _, _, _) => *pretype_evaluate_typ_symbol(db, *target),
    SymbolKind::NamespaceMember(typ_symbol, member_name) => {
      let typ = (*pretype_evaluate_typ_symbol(db, *typ_symbol))?.as_typ()?;
      let lazy = typ.get_namespace_member(db, member_name)?;
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
    VirtualTypKind::MetatypeObj => ObjTyp::new(db).into(),
    VirtualTypKind::MetatypeOString => OStringTyp::new(db).into(),
    VirtualTypKind::MetatypeDeclaration => DeclarationTyp::new(db, None).into(),
    VirtualTypKind::MetatypeQualifiedDeclaration => QualifiedDeclarationTyp::new(db, None).into(),
    VirtualTypKind::MetatypeReference => ReferenceTyp::new(db, None).into(),
    VirtualTypKind::MetatypeQualifiedReference => QualifiedReferenceTyp::new(db, None).into(),
    VirtualTypKind::MetatypeMap => MapTyp::new(db, None, None).into(),
    VirtualTypKind::MetatypeList => ListTyp::new(db, None).into(),
  }
}

/// Creates a virtual FuncObj for a built-in metatype function
fn make_virtual_func_obj<'db>(db: &'db dyn Database, kind: &VirtualFunctionKind) -> FuncObj<'db> {
  let name = match kind {
    VirtualFunctionKind::Interpret => "interpret",
    VirtualFunctionKind::Typeof => "typeof",
  };
  let any = LazyTyp::eager(ObjTyp::new(db).into());
  let signature = FuncSignature::new(db, vec![any], any);
  FuncObj::new(
    db,
    name.to_string(),
    FuncTyp::new(db, signature),
    None,
    None,
  )
}

/// Creates a virtual FuncObj for a built-in operator
fn make_virtual_operator_obj<'db>(
  db: &'db dyn Database,
  kind: &VirtualOperatorKind,
) -> FuncObj<'db> {
  let int = LazyTyp::eager(IntegerTyp::new(db).into());
  let bool = LazyTyp::eager(BoolTyp::new(db).into());
  let string = LazyTyp::eager(StringTyp::new(db).into());
  let obj = LazyTyp::eager(ObjTyp::new(db).into());

  let (params, ret) = match kind {
    // Arithmetic: (int, int) -> int
    VirtualOperatorKind::Add
    | VirtualOperatorKind::Sub
    | VirtualOperatorKind::Mul
    | VirtualOperatorKind::Div
    | VirtualOperatorKind::Mod => (vec![int, int], int),
    // Comparison: (obj, obj) -> bool
    VirtualOperatorKind::Eq
    | VirtualOperatorKind::Neq
    | VirtualOperatorKind::Lt
    | VirtualOperatorKind::Gt
    | VirtualOperatorKind::Lte
    | VirtualOperatorKind::Gte => (vec![obj, obj], bool),
    // Logic: (bool, bool) -> bool
    VirtualOperatorKind::And | VirtualOperatorKind::Or | VirtualOperatorKind::Xor => {
      (vec![bool, bool], bool)
    }
    // String concat: (string, string) -> string
    VirtualOperatorKind::Concat => (vec![string, string], string),
    // Range: (int, int) -> List[int]
    VirtualOperatorKind::Range => {
      let list_int = LazyTyp::eager(ListTyp::new(db, Some(int)).into());
      (vec![int, int], list_int)
    }
    // Coalesce: (obj, obj) -> obj
    VirtualOperatorKind::Coalesce => (vec![obj, obj], obj),
  };

  let signature = FuncSignature::new(db, params, ret);
  FuncObj::new(
    db,
    kind.name().to_string(),
    FuncTyp::new(db, signature),
    None,
    None,
  )
}

/// Dispatches evaluation of a user-declared type node to the alias or element evaluator
fn pretype_evaluate_user_typ_kind<'db>(
  db: &'db dyn Database,
  file: File,
  name: &str,
  node: &RedNode,
) -> Option<Typ<'db>> {
  if let Some(equality_declaration) = EqualityDeclaration::cast(node.clone()) {
    return pretype_evaluate_typ_alias(db, file, &equality_declaration);
  }

  if let Some(block_declaration) = BlockElementDeclaration::cast(node.clone()) {
    return Some(pretype_evaluate_typ_element(
      db,
      file,
      name,
      &block_declaration,
    ));
  }

  None
}
