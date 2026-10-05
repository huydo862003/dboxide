mod any;
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
mod ostring;
mod product;
mod reference;
mod string;
mod sum;
mod typ_typ;
pub mod typecheck;

pub use any::*;
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
pub use ostring::*;
pub use product::*;
pub use reference::*;
pub use string::*;
pub use sum::*;
pub use typ_typ::*;

use std::collections::BTreeMap;

use derive_more::From;
use salsa::Database;

use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::typ_param::TypParams;

/// Runtime value enum, types are first-class values
#[derive(Clone, Copy, PartialEq, Eq, Hash, salsa::SalsaValue, From)]
pub enum Obj<'db> {
  Typ(Typ<'db>),
  String(StringObj<'db>),
  Integer(IntegerObj<'db>),
  Float(FloatObj<'db>),
  Bool(BoolObj<'db>),
  Null(NullObj<'db>),
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
      Obj::Typ(t) => Some(t),
      _ => None,
    }
  }
}

impl<'db> RuntimeObj<'db> for Obj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    match self {
      Obj::Typ(t) => t.get_typ(db),
      Obj::String(o) => o.get_typ(db),
      Obj::Integer(o) => o.get_typ(db),
      Obj::Float(o) => o.get_typ(db),
      Obj::Bool(o) => o.get_typ(db),
      Obj::Null(o) => o.get_typ(db),
      Obj::List(o) => o.get_typ(db),
      Obj::Map(o) => o.get_typ(db),
      Obj::Product(o) => o.get_typ(db),
      Obj::OString(o) => o.get_typ(db),
      Obj::Element(o) => o.get_typ(db),
      Obj::Func(o) => o.get_typ(db),
    }
  }

  fn get_field(&self, db: &'db dyn Database, key: &str) -> Option<Typ<'db>> {
    match self {
      Obj::Typ(t) => t.get_field(db, key),
      Obj::String(o) => o.get_field(db, key),
      Obj::Integer(o) => o.get_field(db, key),
      Obj::Float(o) => o.get_field(db, key),
      Obj::Bool(o) => o.get_field(db, key),
      Obj::Null(o) => o.get_field(db, key),
      Obj::List(o) => o.get_field(db, key),
      Obj::Map(o) => o.get_field(db, key),
      Obj::Product(o) => o.get_field(db, key),
      Obj::OString(o) => o.get_field(db, key),
      Obj::Element(o) => o.get_field(db, key),
      Obj::Func(o) => o.get_field(db, key),
    }
  }

  fn get_owned_field(&self, db: &'db dyn Database, key: &str) -> Option<Obj<'db>> {
    match self {
      Obj::Typ(t) => t.get_owned_field(db, key),
      Obj::String(o) => o.get_owned_field(db, key),
      Obj::Integer(o) => o.get_owned_field(db, key),
      Obj::Float(o) => o.get_owned_field(db, key),
      Obj::Bool(o) => o.get_owned_field(db, key),
      Obj::Null(o) => o.get_owned_field(db, key),
      Obj::List(o) => o.get_owned_field(db, key),
      Obj::Map(o) => o.get_owned_field(db, key),
      Obj::Product(o) => o.get_owned_field(db, key),
      Obj::OString(o) => o.get_owned_field(db, key),
      Obj::Element(o) => o.get_owned_field(db, key),
      Obj::Func(o) => o.get_owned_field(db, key),
    }
  }

  fn lookup_method(&self, db: &'db dyn Database, key: &str) -> Option<FuncObj<'db>> {
    match self {
      Obj::Typ(t) => t.lookup_method(db, key),
      Obj::String(o) => o.lookup_method(db, key),
      Obj::Integer(o) => o.lookup_method(db, key),
      Obj::Float(o) => o.lookup_method(db, key),
      Obj::Bool(o) => o.lookup_method(db, key),
      Obj::Null(o) => o.lookup_method(db, key),
      Obj::List(o) => o.lookup_method(db, key),
      Obj::Map(o) => o.lookup_method(db, key),
      Obj::Product(o) => o.lookup_method(db, key),
      Obj::OString(o) => o.lookup_method(db, key),
      Obj::Element(o) => o.lookup_method(db, key),
      Obj::Func(o) => o.lookup_method(db, key),
    }
  }

  fn lookup_field(&self, db: &'db dyn Database, key: &str) -> Option<Obj<'db>> {
    match self {
      Obj::Typ(t) => t.lookup_field(db, key),
      Obj::String(o) => o.lookup_field(db, key),
      Obj::Integer(o) => o.lookup_field(db, key),
      Obj::Float(o) => o.lookup_field(db, key),
      Obj::Bool(o) => o.lookup_field(db, key),
      Obj::Null(o) => o.lookup_field(db, key),
      Obj::List(o) => o.lookup_field(db, key),
      Obj::Map(o) => o.lookup_field(db, key),
      Obj::Product(o) => o.lookup_field(db, key),
      Obj::OString(o) => o.lookup_field(db, key),
      Obj::Element(o) => o.lookup_field(db, key),
      Obj::Func(o) => o.lookup_field(db, key),
    }
  }

  fn index(&self, db: &'db dyn Database, key: &Obj<'db>) -> Option<Obj<'db>> {
    match self {
      Obj::Typ(t) => t.index(db, key),
      Obj::String(o) => o.index(db, key),
      Obj::Integer(o) => o.index(db, key),
      Obj::Float(o) => o.index(db, key),
      Obj::Bool(o) => o.index(db, key),
      Obj::Null(o) => o.index(db, key),
      Obj::List(o) => o.index(db, key),
      Obj::Map(o) => o.index(db, key),
      Obj::Product(o) => o.index(db, key),
      Obj::OString(o) => o.index(db, key),
      Obj::Element(o) => o.index(db, key),
      Obj::Func(o) => o.index(db, key),
    }
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    match self {
      Obj::Typ(t) => t.to_display_string(db),
      Obj::String(o) => o.to_display_string(db),
      Obj::Integer(o) => o.to_display_string(db),
      Obj::Float(o) => o.to_display_string(db),
      Obj::Bool(o) => o.to_display_string(db),
      Obj::Null(o) => o.to_display_string(db),
      Obj::List(o) => o.to_display_string(db),
      Obj::Map(o) => o.to_display_string(db),
      Obj::Product(o) => o.to_display_string(db),
      Obj::OString(o) => o.to_display_string(db),
      Obj::Element(o) => o.to_display_string(db),
      Obj::Func(o) => o.to_display_string(db),
    }
  }
}

// Allow converting any Typ variant directly to Obj via Typ
macro_rules! impl_from_typ_for_obj {
  ($($variant:ident($ty:ident)),+ $(,)?) => {
    $(
      impl<'db> From<$ty<'db>> for Obj<'db> {
        fn from(v: $ty<'db>) -> Self {
          Obj::Typ(Typ::$variant(v))
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
  Any(AnyTyp),
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, salsa::SalsaValue, From)]
pub enum Typ<'db> {
  TypTyp(TypTyp<'db>),
  String(StringTyp<'db>),
  Integer(IntegerTyp<'db>),
  Float(FloatTyp<'db>),
  Bool(BoolTyp<'db>),
  Null(NullTyp<'db>),
  Color(ColorTyp<'db>),
  Any(AnyTyp<'db>),
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
      Typ::TypTyp(t) => t.$method($db $(, $arg)*),
      Typ::String(t) => t.$method($db $(, $arg)*),
      Typ::Integer(t) => t.$method($db $(, $arg)*),
      Typ::Float(t) => t.$method($db $(, $arg)*),
      Typ::Bool(t) => t.$method($db $(, $arg)*),
      Typ::Null(t) => t.$method($db $(, $arg)*),
      Typ::Color(t) => t.$method($db $(, $arg)*),
      Typ::Any(t) => t.$method($db $(, $arg)*),
      Typ::OString(t) => t.$method($db $(, $arg)*),
      Typ::DateTime(t) => t.$method($db $(, $arg)*),
      Typ::Date(t) => t.$method($db $(, $arg)*),
      Typ::Time(t) => t.$method($db $(, $arg)*),
      Typ::Declaration(t) => t.$method($db $(, $arg)*),
      Typ::QualifiedDeclaration(t) => t.$method($db $(, $arg)*),
      Typ::Reference(t) => t.$method($db $(, $arg)*),
      Typ::QualifiedReference(t) => t.$method($db $(, $arg)*),
      Typ::Map(t) => t.$method($db $(, $arg)*),
      Typ::List(t) => t.$method($db $(, $arg)*),
      Typ::Func(t) => t.$method($db $(, $arg)*),
      Typ::Sum(t) => t.$method($db $(, $arg)*),
      Typ::Lit(t) => t.$method($db $(, $arg)*),
      Typ::Never(t) => t.$method($db $(, $arg)*),
      Typ::MetaElement(t) => t.$method($db $(, $arg)*),
      Typ::Product(t) => t.$method($db $(, $arg)*),
      Typ::Element(t) => t.$method($db $(, $arg)*),
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

  fn static_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, Typ<'db>> {
    delegate_typ!(self, db, static_vtable)
  }

  fn lookup_field_typ(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    delegate_typ!(self, db, lookup_field_typ, name)
  }

  fn lookup_namespace_member(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    delegate_typ!(self, db, lookup_namespace_member, name)
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
}
