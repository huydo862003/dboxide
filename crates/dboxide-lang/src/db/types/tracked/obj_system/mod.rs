pub mod base;
mod bool;
mod color;
mod datetime;
mod declaration;
mod element;
mod float;
mod func;
mod integer;
mod list;
mod lit;
mod map;
mod never;
mod null;
mod obj;
mod ostring;
mod product;
mod reference;
mod string;
mod sum;
mod typ_typ;
pub mod typecheck;

pub use base::*;
pub use bool::*;
pub use color::*;
pub use datetime::*;
pub use declaration::*;
pub use element::*;
pub use float::*;
pub use func::*;
pub use integer::*;
pub use list::*;
pub use lit::*;
pub use map::*;
pub use never::*;
pub use null::*;
pub use obj::*;
pub use ostring::*;
pub use product::*;
pub use reference::*;
pub use string::*;
pub use sum::*;
pub use typ_typ::*;

use std::collections::BTreeMap;

use derive_more::From;
use salsa::{Database, SalsaValue};

use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::typ_param::TypParams;

/// Runtime value enum, types are first-class values
#[derive(Clone, Copy, PartialEq, Eq, Hash, SalsaValue, From)]
pub enum Obj<'db> {
  Typ(Typ<'db>),
  String(StringObj<'db>),
  Integer(IntegerObj<'db>),
  Float(FloatObj<'db>),
  Bool(BoolObj<'db>),
  Null(NullObj<'db>),
  Color(ColorObj<'db>),
  List(ListObj<'db>),
  Map(MapObj<'db>),
  Product(ProductObj<'db>),
  OString(OStringObj<'db>),
  Element(ElementObj<'db>),
  Func(FuncObj<'db>),
}

impl<'db> Obj<'db> {
  pub fn as_typ(self) -> Option<Typ<'db>> {
    match self {
      Obj::Typ(typ) => Some(typ),
      _ => None,
    }
  }

  pub fn as_bool(self, db: &'db dyn Database) -> Option<bool> {
    match self {
      Obj::Bool(bool) => Some(bool.value(db)),
      _ => None,
    }
  }

  /// Coerces a primitive value to a literal type for use as a union member
  pub(crate) fn coerce_to_union_member_typ(self, db: &'db dyn Database) -> Option<Typ<'db>> {
    match self {
      Obj::Typ(typ) => Some(typ),
      Obj::String(s) => Some(Typ::Lit(LitTyp::new(
        db,
        LitValue::String(s.value(db).to_string()),
      ))),
      Obj::Integer(i) => Some(Typ::Lit(LitTyp::new(db, LitValue::Integer(i.value(db))))),
      Obj::Bool(b) => Some(Typ::Lit(LitTyp::new(db, LitValue::Bool(b.value(db))))),
      _ => None,
    }
  }
}

impl<'db> RuntimeObj<'db> for Obj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    match self {
      Obj::Typ(typ) => typ.get_typ(db),
      Obj::String(obj) => obj.get_typ(db),
      Obj::Integer(obj) => obj.get_typ(db),
      Obj::Float(obj) => obj.get_typ(db),
      Obj::Bool(obj) => obj.get_typ(db),
      Obj::Null(obj) => obj.get_typ(db),
      Obj::Color(obj) => obj.get_typ(db),
      Obj::List(obj) => obj.get_typ(db),
      Obj::Map(obj) => obj.get_typ(db),
      Obj::Product(obj) => obj.get_typ(db),
      Obj::OString(obj) => obj.get_typ(db),
      Obj::Element(obj) => obj.get_typ(db),
      Obj::Func(obj) => obj.get_typ(db),
    }
  }

  fn get_field(&self, db: &'db dyn Database, key: &str) -> Option<Typ<'db>> {
    match self {
      Obj::Typ(typ) => typ.get_field(db, key),
      Obj::String(obj) => obj.get_field(db, key),
      Obj::Integer(obj) => obj.get_field(db, key),
      Obj::Float(obj) => obj.get_field(db, key),
      Obj::Bool(obj) => obj.get_field(db, key),
      Obj::Null(obj) => obj.get_field(db, key),
      Obj::Color(obj) => obj.get_field(db, key),
      Obj::List(obj) => obj.get_field(db, key),
      Obj::Map(obj) => obj.get_field(db, key),
      Obj::Product(obj) => obj.get_field(db, key),
      Obj::OString(obj) => obj.get_field(db, key),
      Obj::Element(obj) => obj.get_field(db, key),
      Obj::Func(obj) => obj.get_field(db, key),
    }
  }

  fn get_owned_field(&self, db: &'db dyn Database, key: &str) -> Option<Obj<'db>> {
    match self {
      Obj::Typ(typ) => typ.get_owned_field(db, key),
      Obj::String(obj) => obj.get_owned_field(db, key),
      Obj::Integer(obj) => obj.get_owned_field(db, key),
      Obj::Float(obj) => obj.get_owned_field(db, key),
      Obj::Bool(obj) => obj.get_owned_field(db, key),
      Obj::Null(obj) => obj.get_owned_field(db, key),
      Obj::Color(obj) => obj.get_owned_field(db, key),
      Obj::List(obj) => obj.get_owned_field(db, key),
      Obj::Map(obj) => obj.get_owned_field(db, key),
      Obj::Product(obj) => obj.get_owned_field(db, key),
      Obj::OString(obj) => obj.get_owned_field(db, key),
      Obj::Element(obj) => obj.get_owned_field(db, key),
      Obj::Func(obj) => obj.get_owned_field(db, key),
    }
  }

  fn lookup_method(&self, db: &'db dyn Database, key: &str) -> Option<FuncObj<'db>> {
    match self {
      Obj::Typ(typ) => typ.lookup_method(db, key),
      Obj::String(obj) => obj.lookup_method(db, key),
      Obj::Integer(obj) => obj.lookup_method(db, key),
      Obj::Float(obj) => obj.lookup_method(db, key),
      Obj::Bool(obj) => obj.lookup_method(db, key),
      Obj::Null(obj) => obj.lookup_method(db, key),
      Obj::Color(obj) => obj.lookup_method(db, key),
      Obj::List(obj) => obj.lookup_method(db, key),
      Obj::Map(obj) => obj.lookup_method(db, key),
      Obj::Product(obj) => obj.lookup_method(db, key),
      Obj::OString(obj) => obj.lookup_method(db, key),
      Obj::Element(obj) => obj.lookup_method(db, key),
      Obj::Func(obj) => obj.lookup_method(db, key),
    }
  }

  fn lookup_field(&self, db: &'db dyn Database, key: &str) -> Option<Obj<'db>> {
    match self {
      Obj::Typ(typ) => typ.lookup_field(db, key),
      Obj::String(obj) => obj.lookup_field(db, key),
      Obj::Integer(obj) => obj.lookup_field(db, key),
      Obj::Float(obj) => obj.lookup_field(db, key),
      Obj::Bool(obj) => obj.lookup_field(db, key),
      Obj::Null(obj) => obj.lookup_field(db, key),
      Obj::Color(obj) => obj.lookup_field(db, key),
      Obj::List(obj) => obj.lookup_field(db, key),
      Obj::Map(obj) => obj.lookup_field(db, key),
      Obj::Product(obj) => obj.lookup_field(db, key),
      Obj::OString(obj) => obj.lookup_field(db, key),
      Obj::Element(obj) => obj.lookup_field(db, key),
      Obj::Func(obj) => obj.lookup_field(db, key),
    }
  }

  fn index(&self, db: &'db dyn Database, key: &Obj<'db>) -> Option<Obj<'db>> {
    match self {
      Obj::Typ(typ) => typ.index(db, key),
      Obj::String(obj) => obj.index(db, key),
      Obj::Integer(obj) => obj.index(db, key),
      Obj::Float(obj) => obj.index(db, key),
      Obj::Bool(obj) => obj.index(db, key),
      Obj::Null(obj) => obj.index(db, key),
      Obj::Color(obj) => obj.index(db, key),
      Obj::List(obj) => obj.index(db, key),
      Obj::Map(obj) => obj.index(db, key),
      Obj::Product(obj) => obj.index(db, key),
      Obj::OString(obj) => obj.index(db, key),
      Obj::Element(obj) => obj.index(db, key),
      Obj::Func(obj) => obj.index(db, key),
    }
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    match self {
      Obj::Typ(typ) => typ.to_display_string(db),
      Obj::String(obj) => obj.to_display_string(db),
      Obj::Integer(obj) => obj.to_display_string(db),
      Obj::Float(obj) => obj.to_display_string(db),
      Obj::Bool(obj) => obj.to_display_string(db),
      Obj::Null(obj) => obj.to_display_string(db),
      Obj::Color(obj) => obj.to_display_string(db),
      Obj::List(obj) => obj.to_display_string(db),
      Obj::Map(obj) => obj.to_display_string(db),
      Obj::Product(obj) => obj.to_display_string(db),
      Obj::OString(obj) => obj.to_display_string(db),
      Obj::Element(obj) => obj.to_display_string(db),
      Obj::Func(obj) => obj.to_display_string(db),
    }
  }

  fn lookup_namespace_member(&self, db: &'db dyn Database, name: &str) -> Option<Obj<'db>> {
    match self {
      Obj::Typ(typ) => RuntimeObj::lookup_namespace_member(typ, db, name),
      Obj::Element(obj) => obj.lookup_namespace_member(db, name),
      _ => None,
    }
  }
}

// Allow converting any Typ variant directly to Obj via Typ
macro_rules! impl_from_typ_for_obj {
  ($($variant:ident($ty:ident)),+ $(,)?) => {
    $(
      impl<'db> From<$ty<'db>> for Obj<'db> {
        fn from(value: $ty<'db>) -> Self {
          Obj::Typ(Typ::$variant(value))
        }
      }
    )+
  };
}

impl_from_typ_for_obj!(
  TypTyp(TypTyp),
  String(StringTyp),
  Integer(IntegerTyp),
  Float(FloatTyp),
  Bool(BoolTyp),
  Null(NullTyp),
  Color(ColorTyp),
  Obj(ObjTyp),
  OString(OStringTyp),
  DateTime(DateTimeTyp),
  Date(DateTyp),
  Time(TimeTyp),
  Declaration(DeclarationTyp),
  QualifiedDeclaration(QualifiedDeclarationTyp),
  Reference(ReferenceTyp),
  QualifiedReference(QualifiedReferenceTyp),
  Map(MapTyp),
  List(ListTyp),
  Func(FuncTyp),
  Sum(SumTyp),
  Lit(LitTyp),
  Never(NeverTyp),
  MetaElement(ElementMetaTyp),
  Product(ProductTyp),
  Element(ElementTyp),
);

#[derive(Clone, Copy, PartialEq, Eq, Hash, SalsaValue, From)]
pub enum Typ<'db> {
  TypTyp(TypTyp<'db>),
  String(StringTyp<'db>),
  Integer(IntegerTyp<'db>),
  Float(FloatTyp<'db>),
  Bool(BoolTyp<'db>),
  Null(NullTyp<'db>),
  Color(ColorTyp<'db>),
  Obj(ObjTyp<'db>),
  OString(OStringTyp<'db>),
  DateTime(DateTimeTyp<'db>),
  Date(DateTyp<'db>),
  Time(TimeTyp<'db>),
  Declaration(DeclarationTyp<'db>),
  QualifiedDeclaration(QualifiedDeclarationTyp<'db>),
  Reference(ReferenceTyp<'db>),
  QualifiedReference(QualifiedReferenceTyp<'db>),
  Map(MapTyp<'db>),
  List(ListTyp<'db>),
  Func(FuncTyp<'db>),
  Sum(SumTyp<'db>),
  Lit(LitTyp<'db>),
  Never(NeverTyp<'db>),
  MetaElement(ElementMetaTyp<'db>),
  Product(ProductTyp<'db>),
  Element(ElementTyp<'db>),
}

macro_rules! delegate_typ {
  ($self:expr, $db:expr, $method:ident $(, $arg:expr)*) => {
    match $self {
      Typ::TypTyp(typ) => typ.$method($db $(, $arg)*),
      Typ::String(typ) => typ.$method($db $(, $arg)*),
      Typ::Integer(typ) => typ.$method($db $(, $arg)*),
      Typ::Float(typ) => typ.$method($db $(, $arg)*),
      Typ::Bool(typ) => typ.$method($db $(, $arg)*),
      Typ::Null(typ) => typ.$method($db $(, $arg)*),
      Typ::Color(typ) => typ.$method($db $(, $arg)*),
      Typ::Obj(typ) => typ.$method($db $(, $arg)*),
      Typ::OString(typ) => typ.$method($db $(, $arg)*),
      Typ::DateTime(typ) => typ.$method($db $(, $arg)*),
      Typ::Date(typ) => typ.$method($db $(, $arg)*),
      Typ::Time(typ) => typ.$method($db $(, $arg)*),
      Typ::Declaration(typ) => typ.$method($db $(, $arg)*),
      Typ::QualifiedDeclaration(typ) => typ.$method($db $(, $arg)*),
      Typ::Reference(typ) => typ.$method($db $(, $arg)*),
      Typ::QualifiedReference(typ) => typ.$method($db $(, $arg)*),
      Typ::Map(typ) => typ.$method($db $(, $arg)*),
      Typ::List(typ) => typ.$method($db $(, $arg)*),
      Typ::Func(typ) => typ.$method($db $(, $arg)*),
      Typ::Sum(typ) => typ.$method($db $(, $arg)*),
      Typ::Lit(typ) => typ.$method($db $(, $arg)*),
      Typ::Never(typ) => typ.$method($db $(, $arg)*),
      Typ::MetaElement(typ) => typ.$method($db $(, $arg)*),
      Typ::Product(typ) => typ.$method($db $(, $arg)*),
      Typ::Element(typ) => typ.$method($db $(, $arg)*),
    }
  };
}

impl<'db> StaticTyp<'db> for Typ<'db> {
  fn display_name(&self, db: &'db dyn Database) -> String {
    delegate_typ!(self, db, display_name)
  }

  fn arity(&self, db: &'db dyn Database) -> usize {
    delegate_typ!(self, db, arity)
  }

  fn parent_typ(&self, db: &'db dyn Database) -> Option<Typ<'db>> {
    delegate_typ!(self, db, parent_typ)
  }

  fn is_typ(&self, db: &'db dyn Database) -> bool {
    delegate_typ!(self, db, is_typ)
  }

  fn runtime_typ(&self, db: &'db dyn Database) -> Option<Typ<'db>> {
    delegate_typ!(self, db, runtime_typ)
  }

  fn typ_params(&self, db: &'db dyn Database) -> Option<TypParams<'db>> {
    delegate_typ!(self, db, typ_params)
  }

  fn instantiate(&self, db: &'db dyn Database, args: Vec<LazyTyp<'db>>) -> Option<Typ<'db>> {
    delegate_typ!(self, db, instantiate, args)
  }

  fn construct(&self, db: &'db dyn Database, args: Vec<Typ<'db>>) -> Option<Typ<'db>> {
    delegate_typ!(self, db, construct, args)
  }

  fn get_fields(&self, db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    delegate_typ!(self, db, get_fields)
  }

  fn get_namespace_members(&self, db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    delegate_typ!(self, db, get_namespace_members)
  }

  fn runtime_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, FuncObj<'db>> {
    delegate_typ!(self, db, runtime_vtable)
  }

  fn lookup_runtime_op(&self, db: &'db dyn Database, op: &str) -> Option<FuncObj<'db>> {
    delegate_typ!(self, db, lookup_runtime_op, op)
  }

  fn static_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, Typ<'db>> {
    delegate_typ!(self, db, static_vtable)
  }

  fn lookup_field_typ(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    delegate_typ!(self, db, lookup_field_typ, name)
  }

  fn get_namespace_member(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    delegate_typ!(self, db, get_namespace_member, name)
  }

  fn index_typ(&self, db: &'db dyn Database) -> Option<FuncTyp<'db>> {
    delegate_typ!(self, db, index_typ)
  }

  fn call_typ(&self, db: &'db dyn Database) -> Option<FuncTyp<'db>> {
    delegate_typ!(self, db, call_typ)
  }
}

impl<'db> RuntimeObj<'db> for Typ<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    delegate_typ!(self, db, get_typ)
  }

  fn get_field(&self, db: &'db dyn Database, key: &str) -> Option<Typ<'db>> {
    delegate_typ!(self, db, get_field, key)
  }

  fn get_owned_field(&self, db: &'db dyn Database, key: &str) -> Option<Obj<'db>> {
    delegate_typ!(self, db, get_owned_field, key)
  }

  fn lookup_method(&self, db: &'db dyn Database, key: &str) -> Option<FuncObj<'db>> {
    delegate_typ!(self, db, lookup_method, key)
  }

  fn lookup_field(&self, db: &'db dyn Database, key: &str) -> Option<Obj<'db>> {
    delegate_typ!(self, db, lookup_field, key)
  }

  fn index(&self, db: &'db dyn Database, key: &Obj<'db>) -> Option<Obj<'db>> {
    delegate_typ!(self, db, index, key)
  }

  fn source_path(&self, db: &'db dyn Database) -> String {
    delegate_typ!(self, db, source_path)
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    delegate_typ!(self, db, to_display_string)
  }

  fn lookup_namespace_member(&self, db: &'db dyn Database, name: &str) -> Option<Obj<'db>> {
    let lazy = self.get_namespace_member(db, name)?;
    let resolved = lazy.resolve(db)?;
    Some(Obj::Typ(resolved))
  }
}
