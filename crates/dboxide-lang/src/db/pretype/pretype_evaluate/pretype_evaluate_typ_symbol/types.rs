use std::collections::BTreeMap;

use bitflags::bitflags;
use salsa::Database;

use crate::ast::RedNode;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{
  ConstraintDefinition, ElementTyp, FuncObj, LabelDefinition, OnUsePolicy, Typ,
};

/// Unified evaluated type built from a type body before constructing `ElementTyp`
#[derive(Default)]
pub struct EvaluatedTyp<'db> {
  pub field_declarations: BTreeMap<String, EvaluatedFieldDeclaration<'db>>,
  pub field_subtypes: BTreeMap<String, LazyTyp<'db>>,
  pub element_subtypes: BTreeMap<String, LazyTyp<'db>>,
  pub namespace: BTreeMap<String, LazyTyp<'db>>,
  pub vtable: BTreeMap<String, FuncObj<'db>>,
  pub constraints: BTreeMap<String, ConstraintDefinition>,
  pub on_use: OnUsePolicy,
}

impl<'db> EvaluatedTyp<'db> {
  fn field_typs(&self) -> BTreeMap<String, LazyTyp<'db>> {
    self
      .field_declarations
      .iter()
      .map(|(k, f)| (k.clone(), f.typ))
      .collect()
  }

  fn labels(&self) -> Vec<LabelDefinition> {
    let mut labeled: Vec<_> = self
      .field_declarations
      .iter()
      .filter_map(|(name, field)| {
        field.label.map(|flags| LabelDefinition {
          field_name: name.clone(),
          is_alias: flags.contains(LabelFlags::ALIAS),
          unique: flags.contains(LabelFlags::UNIQUE),
          exposes_namespace: flags.contains(LabelFlags::EXPOSES_NAMESPACE),
        })
      })
      .collect();
    labeled.sort_by(|a, b| a.field_name.cmp(&b.field_name));
    labeled
  }

  fn attributes(&self) -> BTreeMap<String, LazyTyp<'db>> {
    self
      .field_declarations
      .iter()
      .filter(|(_, f)| f.flags.contains(FieldAnnotationFlags::ATTRIBUTE))
      .map(|(k, f)| (k.clone(), f.typ))
      .collect()
  }

  fn settings(&self) -> BTreeMap<String, LazyTyp<'db>> {
    self
      .field_declarations
      .iter()
      .filter(|(_, f)| f.setting_key.is_some())
      .map(|(k, f)| (k.clone(), f.typ))
      .collect()
  }

  pub fn into_element_typ(self, db: &'db dyn Database, name: String) -> Typ<'db> {
    ElementTyp::new(
      db,
      name,
      self.field_typs(),
      self.labels(),
      self.attributes(),
      self.settings(),
      self.field_subtypes,
      self.element_subtypes,
      self.namespace,
      self.vtable,
      self.constraints,
      self.on_use,
      None,
    )
    .into()
  }
}

/// A single-line declaration inside a type body (`name Type [annotations]`)
pub struct EvaluatedFieldDeclaration<'db> {
  pub typ: LazyTyp<'db>,
  pub arg_position: Option<ArgPosition>,
  pub flags: FieldAnnotationFlags,
  pub label: Option<LabelFlags>,
  /// `None` = not a setting
  /// `Some(None)` = `[setting]`
  /// `Some(Some(key))` = `[setting: key]`
  pub setting_key: Option<Option<String>>,
  pub node: RedNode,
}

impl EvaluatedFieldDeclaration<'_> {
  pub fn span(&self) -> (usize, usize) {
    let start = self.node.offset();
    (start, start + self.node.text_len())
  }
}

bitflags! {
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
  pub struct FieldAnnotationFlags: u8 {
    const CSV       = 1 << 0;
    /// `[attribute]`: controls how this field appears in element **instance** syntax, not in the type body.
    /// - On a `[field]` type: the field is written as a bare keyword inline (e.g. `pk` in `id integer pk`)
    /// - On an `[element]` type: the field is written as `key: value` in the element body
    /// May coexist with `[setting]` if the field can appear in either position.
    const ATTRIBUTE = 1 << 1;
  }
}

bitflags! {
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
  pub struct LabelFlags: u8 {
    /// `[label]` - plain label (mutually exclusive with `ALIAS`)
    const LABEL             = 1 << 0;
    /// `[label alias]` - alias label (mutually exclusive with `LABEL`)
    const ALIAS             = 1 << 1;
    const UNIQUE            = 1 << 2;
    const EXPOSES_NAMESPACE = 1 << 3;
  }
}

/// Positional argument index: exact `[arg: n]` or variadic `[arg: n..]`
pub enum ArgPosition {
  Exact(usize),
  Variadic(usize),
}

impl ArgPosition {
  pub fn index(&self) -> usize {
    match self {
      ArgPosition::Exact(n) | ArgPosition::Variadic(n) => *n,
    }
  }
}

/// Parsed constraint entry (name + optional closure) from a constraint block attribute
pub struct EvaluatedConstraintEntry {
  pub name: String,
  pub closure_syntax: Option<RedNode>,
  pub node: RedNode,
}

impl EvaluatedConstraintEntry {
  pub fn span(&self) -> (usize, usize) {
    let start = self.node.offset();
    (start, start + self.node.text_len())
  }
}

/// A non-type nested block element found inside a type body
pub struct EvaluatedNestedElement {
  pub element_typ: String,
  pub node: RedNode,
}

impl EvaluatedNestedElement {
  pub fn span(&self) -> (usize, usize) {
    let start = self.node.offset();
    (start, start + self.node.text_len())
  }
}
