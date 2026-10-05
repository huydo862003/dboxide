use std::collections::HashMap;

use salsa::interned;

use crate::ast::{AstNode, FnDeclaration, RedNode};
use crate::db::types::input::File;
use crate::db::types::tracked::obj_system::Typ;

#[interned]
pub struct Symbol<'db> {
  #[returns(ref)]
  pub kind: SymbolKind<'db>,
  #[returns(deref)]
  pub name: String,
  /// Fully qualified path, e.g. "/path/to/file.dbml::TableName"
  #[returns(deref)]
  pub definition_id: String,
}

#[derive(Clone, PartialEq, Eq, Hash, salsa::SalsaValue)]
pub enum SymbolKind<'db> {
  VirtualTyp(VirtualModuleKind, VirtualTypKind),
  VirtualModule(VirtualModuleKind),
  VirtualFunction(VirtualModuleKind, VirtualFunctionKind),
  UserTyp(File, RedNode),
  UserModule(File, RedNode),
  /// Implicit schema namespace. Parent is None for top-level schemas.
  UserSchema(File, Option<Symbol<'db>>),
  UserFunction(File, RedNode),
  UserOperator(File, RedNode),
  UserElement(File, RedNode),
  /// `Table users as u`
  ElementAlias(Symbol<'db>),
  /// `use { table users } from '...'`
  Use(Symbol<'db>, File, RedNode),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolModule {
  Virtual(VirtualModuleKind),
  File(File),
}

impl<'db> SymbolKind<'db> {
  pub fn is_virtual(&self) -> bool {
    matches!(
      self,
      SymbolKind::VirtualTyp(..) | SymbolKind::VirtualModule(_) | SymbolKind::VirtualFunction(..)
    )
  }

  pub fn is_user_defined(&self) -> bool {
    !self.is_virtual() && !matches!(self, SymbolKind::ElementAlias(_) | SymbolKind::Use(..))
  }
}

#[derive(Clone, PartialEq, Eq, Hash, salsa::SalsaValue)]
pub enum SymbolCategory<'db> {
  Typ,
  Module,
  Function,
  Operator(Symbol<'db>),
  Schema,
  Element(Symbol<'db>),
  Alias(Symbol<'db>),
}

#[derive(Clone, PartialEq, Eq, Hash, salsa::SalsaValue)]
pub struct SymbolKey<'db>(pub String, pub SymbolCategory<'db>);

#[derive(Clone, Default, PartialEq, Eq, salsa::SalsaValue)]
pub struct StaticScopeMembers<'db>(HashMap<SymbolKey<'db>, Symbol<'db>>);

impl<'db> StaticScopeMembers<'db> {
  pub fn lookup_by_name<'a>(
    &'a self,
    name: &str,
  ) -> impl Iterator<Item = (&'a SymbolKey<'db>, &'a Symbol<'db>)>
  where
    'db: 'a,
  {
    self.0.iter().filter(move |(key, _)| key.0 == name)
  }

  pub fn insert(&mut self, db: &'db dyn salsa::Database, symbol: Symbol<'db>) {
    self.0.insert(symbol.stable_key(db), symbol);
  }

  pub fn extend(&mut self, other: StaticScopeMembers<'db>) {
    self.0.extend(other.0);
  }

  pub fn iter(&self) -> impl Iterator<Item = (&SymbolKey<'db>, &Symbol<'db>)> {
    self.0.iter()
  }
}

impl<'db> Symbol<'db> {
  pub fn category(&self, db: &'db dyn salsa::Database) -> SymbolCategory<'db> {
    match self.kind(db) {
      SymbolKind::VirtualTyp(..) | SymbolKind::UserTyp(..) => SymbolCategory::Typ,
      SymbolKind::VirtualModule(..) | SymbolKind::UserModule(..) => SymbolCategory::Module,
      SymbolKind::VirtualFunction(..) | SymbolKind::UserFunction(..) => SymbolCategory::Function,
      SymbolKind::UserOperator(..) => SymbolCategory::Operator(*self),
      SymbolKind::UserSchema(..) => SymbolCategory::Schema,
      SymbolKind::UserElement(..) => SymbolCategory::Element(*self),
      SymbolKind::ElementAlias(original) => SymbolCategory::Alias(*original),
      SymbolKind::Use(original, _, _) => original.category(db),
    }
  }

  pub fn stable_key(&self, db: &'db dyn salsa::Database) -> SymbolKey<'db> {
    SymbolKey(self.name(db).to_string(), self.category(db))
  }

  pub fn resolve_operator_signature(
    &self,
    db: &'db dyn salsa::Database,
  ) -> Option<(Vec<Typ<'db>>, Option<Typ<'db>>)> {
    let SymbolKind::UserOperator(file, node) = self.kind(db) else {
      return None;
    };
    let func = FnDeclaration::cast(node.clone())?;
    let mut param_typs = vec![];
    if let Some(params) = func.params() {
      for param in params.params() {
        if let Some(type_expr) = param.type_expr() {
          let obj = crate::db::pretype::comp_evaluate_node(db, *file, &type_expr);
          if let Some(typ) = obj.and_then(|o| o.as_typ()) {
            param_typs.push(typ);
          }
        }
      }
    }
    let return_typ = func
      .return_typ()
      .and_then(|rt| rt.type_expr())
      .and_then(|type_expr| crate::db::pretype::comp_evaluate_node(db, *file, &type_expr))
      .and_then(|o| o.as_typ());
    Some((param_typs, return_typ))
  }

  /// Follow aliases and use indirections to the original symbol
  pub fn original_symbol(&self, db: &'db dyn salsa::Database) -> Symbol<'db> {
    let mut current = *self;
    let mut visited = hashbrown::HashSet::new();
    loop {
      match current.kind(db) {
        SymbolKind::ElementAlias(original) | SymbolKind::Use(original, _, _) => {
          if !visited.insert(current) {
            return current;
          }
          current = *original;
        }
        _ => return current,
      }
    }
  }

  /// The name before any aliasing
  pub fn original_name(&self, db: &'db dyn salsa::Database) -> &'db str {
    self.original_symbol(db).name(db)
  }

  /// The module this symbol was originally defined in
  pub fn original_module(&self, db: &'db dyn salsa::Database) -> Option<SymbolModule> {
    match self.original_symbol(db).kind(db) {
      SymbolKind::VirtualTyp(module, _)
      | SymbolKind::VirtualModule(module)
      | SymbolKind::VirtualFunction(module, _) => Some(SymbolModule::Virtual(*module)),
      SymbolKind::UserTyp(file, _)
      | SymbolKind::UserModule(file, _)
      | SymbolKind::UserSchema(file, _)
      | SymbolKind::UserFunction(file, _)
      | SymbolKind::UserOperator(file, _)
      | SymbolKind::UserElement(file, _) => Some(SymbolModule::File(*file)),
      SymbolKind::ElementAlias(_) | SymbolKind::Use(..) => {
        debug_assert!(false, "original_symbol returned Alias/Use, likely a cycle");
        None
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VirtualModuleKind {
  Metatype,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VirtualTypKind {
  MetatypeString,
  MetatypeInteger,
  MetatypeFloat,
  MetatypeBool,
  MetatypeNull,
  MetatypeColor,
  MetatypeAny,
  /// Opaque string, passed through verbatim
  MetatypeOString,
  /// `declaration[T]`
  MetatypeDeclaration,
  /// `qualified_declaration[T]`
  MetatypeQualifiedDeclaration,
  /// `reference[T]`
  MetatypeReference,
  /// `qualified_reference[T]`
  MetatypeQualifiedReference,
  /// `Map[K, V]`
  MetatypeMap,
  /// `List[V]`
  MetatypeList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VirtualFunctionKind {
  Interpret,
  Typeof,
}
