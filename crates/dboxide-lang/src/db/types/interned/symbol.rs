use std::collections::HashMap;

use salsa::{Database, SalsaValue, interned};

use crate::DEFAULT_SCHEMA;
use crate::ast::{AstNode, ElementDeclaration, RedNode, UseSpecifier};
use crate::db::types::input::File;

#[interned]
pub struct Symbol<'db> {
  #[returns(ref)]
  pub kind: SymbolKind<'db>,
}

#[derive(Clone, PartialEq, Eq, Hash, SalsaValue)]
pub enum SymbolKind<'db> {
  VirtualTyp(VirtualModuleKind, VirtualTypKind),
  VirtualModule(VirtualModuleKind),
  VirtualFunction(VirtualModuleKind, VirtualFunctionKind),
  VirtualOperator(VirtualOperatorKind),
  UserTyp(File, RedNode, String),
  UserModule(File, RedNode, String),
  /// Implicit schema namespace
  /// Parent is None for top-level schemas
  UserSchema(File, Option<Symbol<'db>>, String),
  UserFunction(File, RedNode, String),
  UserOperator(File, RedNode, String),
  UserElement(File, RedNode, String),
  /// Unlabeled element instance (no name) - participates in wildcard imports
  AnonymousElement(File, RedNode),
  /// `Table users as u`
  ElementAlias(Symbol<'db>, String),
  /// `use { table users } from '...'`
  Use(Symbol<'db>, File, RedNode, String),
  /// A namespace member of a type with `[use: expand namespaces]`, exposed directly into scope
  /// Inner symbol is the type, string is the member name
  NamespaceMember(Symbol<'db>, String),
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
      SymbolKind::VirtualTyp(..)
        | SymbolKind::VirtualModule(_)
        | SymbolKind::VirtualFunction(..)
        | SymbolKind::VirtualOperator(..)
    )
  }

  pub fn is_user_defined(&self) -> bool {
    !self.is_virtual()
      && !matches!(
        self,
        SymbolKind::ElementAlias(_, _)
          | SymbolKind::Use(..)
          | SymbolKind::NamespaceMember(..)
          | SymbolKind::AnonymousElement(..)
      )
  }
}

/// Filter used in `lookup_symbols` to match symbols by kind; mirrors `ScopeSlot` without payloads
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SymbolCategoryKind {
  Typ,
  Module,
  Function,
  Operator,
  Schema,
  Element,
}

/// Discriminator in the scope HashMap key; carries payload to ensure uniqueness within a name
#[derive(Clone, PartialEq, Eq, Hash, SalsaValue)]
pub enum ScopeSlot<'db> {
  Typ,
  Module,
  Function,
  Operator(Symbol<'db>),
  Schema,
  Element(Symbol<'db>),
  Alias(Symbol<'db>),
  Use(Symbol<'db>),
}

#[derive(Clone, PartialEq, Eq, Hash, SalsaValue)]
pub struct SymbolKey<'db>(pub String, pub ScopeSlot<'db>);

#[derive(Clone, Default, PartialEq, Eq, SalsaValue)]
pub struct StaticScopeMembers<'db>(HashMap<SymbolKey<'db>, Symbol<'db>>);

impl<'db> StaticScopeMembers<'db> {
  pub fn lookup_by_name<'a>(
    &'a self,
    name: &str,
  ) -> impl Iterator<Item = (&'a SymbolKey<'db>, &'a Symbol<'db>)>
  where
    'db: 'a,
  {
    self
      .0
      .iter()
      .filter(move |(key, _)| key.0.eq_ignore_ascii_case(name))
  }

  pub fn lookup_symbols<'a>(
    &'a self,
    db: &'db dyn Database,
    name: &'a str,
    kind: SymbolCategoryKind,
  ) -> impl Iterator<Item = Symbol<'db>> + 'a
  where
    'db: 'a,
  {
    self
      .0
      .iter()
      .filter(move |(key, _)| key.0.eq_ignore_ascii_case(name))
      .map(|(_, symbol)| *symbol)
      .filter(move |symbol| symbol.variant_kind(db) == kind)
  }

  pub fn insert(&mut self, db: &'db dyn Database, symbol: Symbol<'db>) {
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
  /// Returns the symbol's name; for `AnonymousElement` it is derived lazily from kind fields
  pub fn name(&self, db: &'db dyn Database) -> String {
    match self.kind(db) {
      SymbolKind::VirtualTyp(_, kind) => kind.name().to_string(),
      SymbolKind::VirtualModule(kind) => kind.name().to_string(),
      SymbolKind::VirtualFunction(_, kind) => kind.name().to_string(),
      SymbolKind::VirtualOperator(kind) => kind.name().to_string(),
      SymbolKind::UserTyp(_, _, name)
      | SymbolKind::UserModule(_, _, name)
      | SymbolKind::UserSchema(_, _, name)
      | SymbolKind::UserFunction(_, _, name)
      | SymbolKind::UserOperator(_, _, name)
      | SymbolKind::UserElement(_, _, name)
      | SymbolKind::ElementAlias(_, name)
      | SymbolKind::Use(_, _, _, name)
      | SymbolKind::NamespaceMember(_, name) => name.clone(),
      SymbolKind::AnonymousElement(file, node) => {
        let filepath = file.filepath(db).display().to_string();
        let typ_qualified_name = ElementDeclaration::cast(node.clone())
          .and_then(|element| element.typ())
          .map(|typ| {
            let schema_parts: Vec<String> = typ
              .schema()
              .into_iter()
              .map(|fragment| fragment.name().to_string())
              .collect();
            let typ_name = typ
              .name()
              .map(|fragment| fragment.name().to_string())
              .unwrap_or_default();
            if schema_parts.is_empty() {
              format!("{DEFAULT_SCHEMA}.{typ_name}")
            } else {
              format!("{}.{typ_name}", schema_parts.join("."))
            }
          })
          .unwrap_or_default();
        format!("@anonymous::{filepath}::{typ_qualified_name}")
      }
    }
  }

  /// Derives a fully qualified path string from the symbol's kind and name
  pub fn definition_id(&self, db: &'db dyn Database) -> String {
    let name = self.name(db);
    match self.kind(db) {
      SymbolKind::VirtualTyp(module, _)
      | SymbolKind::VirtualModule(module)
      | SymbolKind::VirtualFunction(module, _) => {
        let module_name = match module {
          VirtualModuleKind::Metatype => "metatype",
        };
        format!("@{module_name}::{name}")
      }
      SymbolKind::VirtualOperator(_) => format!("@builtin::{name}"),
      SymbolKind::UserTyp(file, node, _)
      | SymbolKind::UserModule(file, node, _)
      | SymbolKind::UserFunction(file, node, _)
      | SymbolKind::UserOperator(file, node, _)
      | SymbolKind::UserElement(file, node, _) => {
        let filepath = file.filepath(db).display().to_string();
        let offset = node.start_offset();
        format!("{filepath}::{offset}::{name}")
      }
      SymbolKind::AnonymousElement(file, node) => {
        let filepath = file.filepath(db).display().to_string();
        let offset = node.start_offset();
        format!("{filepath}::@anonymous::{offset}")
      }
      SymbolKind::UserSchema(file, parent, _) => {
        let filepath = file.filepath(db).display().to_string();
        match parent {
          Some(parent) => format!("{}::{name}", parent.definition_id(db)),
          None => format!("{filepath}::@schema::{name}"),
        }
      }
      SymbolKind::ElementAlias(inner, _) => inner.definition_id(db),
      SymbolKind::Use(inner, _, _, _) => inner.definition_id(db),
      SymbolKind::NamespaceMember(inner, member) => {
        format!("{}::@ns::{member}", inner.definition_id(db))
      }
    }
  }

  pub fn variant_kind(&self, db: &'db dyn Database) -> SymbolCategoryKind {
    match self.kind(db) {
      SymbolKind::VirtualTyp(..) | SymbolKind::UserTyp(..) | SymbolKind::NamespaceMember(..) => {
        SymbolCategoryKind::Typ
      }
      SymbolKind::VirtualModule(..) | SymbolKind::UserModule(..) => SymbolCategoryKind::Module,
      SymbolKind::VirtualFunction(..) | SymbolKind::UserFunction(..) => {
        SymbolCategoryKind::Function
      }
      SymbolKind::VirtualOperator(..) | SymbolKind::UserOperator(..) => {
        SymbolCategoryKind::Operator
      }
      SymbolKind::UserSchema(..) => SymbolCategoryKind::Schema,
      SymbolKind::UserElement(..) | SymbolKind::AnonymousElement(..) => SymbolCategoryKind::Element,
      SymbolKind::ElementAlias(inner, _) | SymbolKind::Use(inner, _, _, _) => {
        inner.variant_kind(db)
      }
    }
  }

  pub fn scope_slot(&self, db: &'db dyn Database) -> ScopeSlot<'db> {
    match self.kind(db) {
      SymbolKind::VirtualTyp(..) | SymbolKind::UserTyp(..) => ScopeSlot::Typ,
      SymbolKind::VirtualModule(..) | SymbolKind::UserModule(..) => ScopeSlot::Module,
      SymbolKind::VirtualFunction(..) | SymbolKind::UserFunction(..) => ScopeSlot::Function,
      SymbolKind::VirtualOperator(..) | SymbolKind::UserOperator(..) => ScopeSlot::Operator(*self),
      SymbolKind::UserSchema(..) => ScopeSlot::Schema,
      SymbolKind::UserElement(..) | SymbolKind::AnonymousElement(..) => ScopeSlot::Element(*self),
      SymbolKind::ElementAlias(original, _) => ScopeSlot::Alias(*original),
      SymbolKind::Use(original, _, _, _) => ScopeSlot::Use(*original),
      SymbolKind::NamespaceMember(..) => ScopeSlot::Typ,
    }
  }

  pub fn stable_key(&self, db: &'db dyn Database) -> SymbolKey<'db> {
    SymbolKey(self.name(db).to_string(), self.scope_slot(db))
  }

  /// Returns the fully qualified name as `[schema..., name]`
  pub fn schema_qualified_name(&self, db: &'db dyn Database) -> Vec<String> {
    let name = self.name(db).to_string();
    match self.kind(db) {
      // Walk the parent chain: ["auth", "sub"] for auth.sub
      SymbolKind::UserSchema(_, _, _) => {
        let mut chain = vec![name];
        let mut current = *self;
        while let SymbolKind::UserSchema(_, Some(parent), _) = current.kind(db) {
          chain.push(parent.name(db).to_string());
          current = *parent;
        }
        chain.reverse();
        chain
      }
      // Schema from AST node; falls back to DEFAULT_SCHEMA
      SymbolKind::UserTyp(_, node, _)
      | SymbolKind::UserModule(_, node, _)
      | SymbolKind::UserFunction(_, node, _)
      | SymbolKind::UserOperator(_, node, _)
      | SymbolKind::UserElement(_, node, _) => {
        let mut schema: Vec<String> = ElementDeclaration::cast(node.clone())
          .and_then(|element| element.typ())
          .map(|typ| {
            typ
              .schema()
              .into_iter()
              .map(|fragment| fragment.name().to_string())
              .collect()
          })
          .unwrap_or_default();
        if schema.is_empty() {
          vec![DEFAULT_SCHEMA.to_string(), name]
        } else {
          schema.push(name);
          schema
        }
      }
      // name is derived via computed_name: "@anonymous::<filepath>::<qualified-type-name>"
      SymbolKind::AnonymousElement(_, _) => {
        vec![DEFAULT_SCHEMA.to_string(), self.name(db).to_string()]
      }
      // `as alias` always lives in DEFAULT_SCHEMA
      SymbolKind::ElementAlias(_, _) => {
        vec![DEFAULT_SCHEMA.to_string(), name]
      }
      // Schema from specifier name path (e.g. `auth` in `use { auth.users }`)
      // falls back to original's schema for wildcard imports
      SymbolKind::Use(original, _, node, _) => {
        let schema: Vec<String> = UseSpecifier::cast(node.clone())
          .and_then(|schema| schema.name())
          .map(|fragment| {
            let path = fragment.path();
            path[..path.len().saturating_sub(1)]
              .iter()
              .map(|fragment| fragment.name().to_string())
              .collect()
          })
          .unwrap_or_else(|| {
            let mut qualified_name = original.schema_qualified_name(db);
            qualified_name.pop();
            qualified_name
          });
        if schema.is_empty() {
          vec![DEFAULT_SCHEMA.to_string(), name]
        } else {
          let mut result = schema;
          result.push(name);
          result
        }
      }
      // Owner's full qualified name + own name
      SymbolKind::NamespaceMember(owner, _) => {
        let mut owner_qualified = owner.schema_qualified_name(db);
        owner_qualified.push(name);
        owner_qualified
      }
      SymbolKind::VirtualTyp(..)
      | SymbolKind::VirtualModule(_)
      | SymbolKind::VirtualFunction(..)
      | SymbolKind::VirtualOperator(..) => {
        vec![DEFAULT_SCHEMA.to_string(), name]
      }
    }
  }

  /// Follow aliases and use indirections to the original symbol
  pub fn original_symbol(&self, db: &'db dyn Database) -> Symbol<'db> {
    let mut current = *self;
    let mut visited = hashbrown::HashSet::new();
    loop {
      match current.kind(db) {
        SymbolKind::ElementAlias(original, _) | SymbolKind::Use(original, _, _, _) => {
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
  pub fn original_name(&self, db: &'db dyn Database) -> String {
    self.original_symbol(db).name(db)
  }

  /// The module this symbol was originally defined in
  pub fn original_module(&self, db: &'db dyn Database) -> Option<SymbolModule> {
    match self.original_symbol(db).kind(db) {
      SymbolKind::VirtualTyp(module, _)
      | SymbolKind::VirtualModule(module)
      | SymbolKind::VirtualFunction(module, _) => Some(SymbolModule::Virtual(*module)),
      SymbolKind::VirtualOperator(..) => None,
      SymbolKind::UserTyp(file, _, _)
      | SymbolKind::UserModule(file, _, _)
      | SymbolKind::UserSchema(file, _, _)
      | SymbolKind::UserFunction(file, _, _)
      | SymbolKind::UserOperator(file, _, _)
      | SymbolKind::UserElement(file, _, _)
      | SymbolKind::AnonymousElement(file, _) => Some(SymbolModule::File(*file)),
      SymbolKind::ElementAlias(_, _) | SymbolKind::Use(..) => {
        debug_assert!(false, "original_symbol returned Alias/Use, likely a cycle");
        None
      }
      SymbolKind::NamespaceMember(inner, _) => inner.original_module(db),
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
  MetatypeObj,
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

impl VirtualModuleKind {
  pub fn name(self) -> &'static str {
    match self {
      Self::Metatype => "metatype",
    }
  }
}

impl VirtualTypKind {
  pub fn name(self) -> &'static str {
    match self {
      Self::MetatypeString => "string",
      Self::MetatypeInteger => "integer",
      Self::MetatypeFloat => "float",
      Self::MetatypeBool => "bool",
      Self::MetatypeNull => "null",
      Self::MetatypeColor => "color",
      Self::MetatypeObj => "obj",
      Self::MetatypeOString => "ostring",
      Self::MetatypeDeclaration => "declaration",
      Self::MetatypeQualifiedDeclaration => "qualified_declaration",
      Self::MetatypeReference => "reference",
      Self::MetatypeQualifiedReference => "qualified_reference",
      Self::MetatypeMap => "Map",
      Self::MetatypeList => "List",
    }
  }
}

impl VirtualFunctionKind {
  pub fn name(self) -> &'static str {
    match self {
      Self::Interpret => "interpret",
      Self::Typeof => "typeof",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VirtualOperatorKind {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Eq,
  Neq,
  Lt,
  Gt,
  Lte,
  Gte,
  And,
  Or,
  Xor,
  Concat,
  Range,
  Coalesce,
}

impl VirtualOperatorKind {
  pub fn name(self) -> &'static str {
    match self {
      Self::Add => "+",
      Self::Sub => "-",
      Self::Mul => "*",
      Self::Div => "/",
      Self::Mod => "%",
      Self::Eq => "==",
      Self::Neq => "!=",
      Self::Lt => "<",
      Self::Gt => ">",
      Self::Lte => "<=",
      Self::Gte => ">=",
      Self::And => "&&",
      Self::Or => "||",
      Self::Xor => "^",
      Self::Concat => "++",
      Self::Range => "..",
      Self::Coalesce => "??",
    }
  }

  pub fn all() -> &'static [VirtualOperatorKind] {
    &[
      Self::Add,
      Self::Sub,
      Self::Mul,
      Self::Div,
      Self::Mod,
      Self::Eq,
      Self::Neq,
      Self::Lt,
      Self::Gt,
      Self::Lte,
      Self::Gte,
      Self::And,
      Self::Or,
      Self::Xor,
      Self::Concat,
      Self::Range,
      Self::Coalesce,
    ]
  }
}
