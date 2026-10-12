use std::collections::BTreeMap;

use salsa::Database;

use crate::ast::RedNode;
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{
  ArgFieldDefinition, AttributeSchema, ElementTyp, FuncObj, OnUsePolicy, Typ, TypElementLabel,
};
use crate::evaluate_typ::types::{ArgPosition, FieldAnnotationFlags, LabelFlags};
use crate::{AnyTyp, TypConstraint, TypNestedTyps, TypSettings};

/* Partially processed type */
#[derive(Default)]
pub(super) struct EvaluatedTyp<'db> {
  pub vtable: BTreeMap<String, FuncObj<'db>>,

  pub typ_settings: EvaluatedTypSettings, // Type name [on use: expand, element, field]
  pub typ_field_declarations: BTreeMap<String, EvaluatedTypField<'db>>, // field-name field-type [settings]
  pub typ_rest_attribute: Option<LazyTyp<'db>>,                         // [rest-names] type
  pub typ_nested_typs: EvaluatedTypNestedTyps<'db>, // { Type name [element/field] }
  pub typ_namespace: BTreeMap<String, LazyTyp<'db>>, // { namespace {} }

  pub typ_constraints: BTreeMap<String, TypConstraint>, // { constraints {} }
}

impl<'db> EvaluatedTyp<'db> {
  pub fn into_element_typ(self, db: &'db dyn Database, name: String) -> Typ<'db> {
    let attributes = self.typ_element_attributes();
    let labels = self.typ_element_labels();
    let settings = self.typ_element_settings();
    let arg_fields = self.typ_element_arg_fields();
    let constraints = self.typ_constraints;
    ElementTyp::new(
      db,
      name,
      self.vtable,
      Some(AnyTyp::new(db).into()),
      TypSettings {
        on_use: self.typ_settings.on_use,
        is_field_typ: self.typ_settings.is_field_typ,
        is_element_typ: self.typ_settings.is_element_typ,
      },
      self
        .typ_field_declarations
        .into_iter()
        .map(|(key, value)| (key, value.typ))
        .collect(),
      TypNestedTyps {
        field_nested_typs: self.typ_nested_typs.field_nested_typs,
        element_nested_typs: self.typ_nested_typs.element_nested_typs,
      },
      self.typ_namespace,
      constraints,
      labels,
      settings,
      arg_fields,
      attributes,
    )
    .into()
  }

  fn typ_element_labels(&self) -> Vec<TypElementLabel> {
    let mut labeled: Vec<_> = self
      .typ_field_declarations
      .iter()
      .filter_map(|(name, field)| {
        field.label.map(|flags| TypElementLabel {
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

  fn typ_element_arg_fields(&self) -> BTreeMap<usize, ArgFieldDefinition<'db>> {
    self
      .typ_field_declarations
      .iter()
      .filter_map(|(name, field)| {
        field.arg_position.as_ref().map(|pos| {
          (
            pos.index(),
            ArgFieldDefinition {
              name: name.clone(),
              typ: field.typ,
              is_variadic: matches!(pos, ArgPosition::Variadic(_)),
              has_csv: field.flags.contains(FieldAnnotationFlags::CSV),
            },
          )
        })
      })
      .collect()
  }

  fn typ_element_attributes(&self) -> AttributeSchema<'db> {
    let named = self
      .typ_field_declarations
      .iter()
      .filter(|(_, field)| field.flags.contains(FieldAnnotationFlags::ATTRIBUTE))
      .map(|(key, field)| (key.clone(), field.typ))
      .collect();
    AttributeSchema {
      named,
      rest: self.typ_rest_attribute,
    }
  }

  fn typ_element_settings(&self) -> BTreeMap<String, LazyTyp<'db>> {
    self
      .typ_field_declarations
      .iter()
      .filter(|(_, field)| field.setting_key.is_some())
      .map(|(key, field)| (key.clone(), field.typ))
      .collect()
  }
}

/* Partially processed type settings (Type name [...]) */
#[derive(Default)]
pub(super) struct EvaluatedTypSettings {
  pub on_use: OnUsePolicy,
  pub is_element_typ: bool,
  pub is_field_typ: bool,
}

/* Partially processed type field line (field-name field-type [settings]) */
pub(in crate::db::pretype::evaluate_typ) struct EvaluatedTypField<'db> {
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

impl EvaluatedTypField<'_> {
  pub fn span(&self) -> (usize, usize) {
    let start = self.node.offset();
    (start, start + self.node.text_len())
  }
}

/* Partially processed type nested types */
#[derive(Default)]
pub(super) struct EvaluatedTypNestedTyps<'db> {
  pub field_nested_typs: BTreeMap<String, LazyTyp<'db>>, // { Type name [field] {} }
  pub element_nested_typs: BTreeMap<String, LazyTyp<'db>>, // { Type name [element] {} }
}

/* Partially processed type constraint */
pub(in crate::db::pretype::evaluate_typ) struct EvaluatedConstraintEntry {
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
