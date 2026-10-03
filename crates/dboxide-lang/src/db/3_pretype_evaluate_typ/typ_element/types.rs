use std::collections::{BTreeMap, BTreeSet};

use salsa::Database;

use crate::ast::RedNode;
use crate::db::pretype_evaluate_typ::types::{ArgPosition, FieldAnnotationFlags};
use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::{
  ArgFieldDefinition, AttributeSchema, ElementTyp, FuncObj, OnUsePolicy, Typ, TypElementLabel,
  TypElementMeta, TypNamespace,
};
use crate::{ObjTyp, TypConstraint, TypNestedTyps, TypSettings};

/* Partially processed type */
#[derive(Default)]
pub(super) struct EvaluatedTyp<'db> {
  pub vtable: BTreeMap<String, FuncObj<'db>>,

  pub typ_settings: EvaluatedTypSettings, // Type name [use: expand namespaces, element, field]
  pub typ_field_declarations: BTreeMap<String, EvaluatedTypField<'db>>, // field-name field-type [settings]
  pub typ_rest_attribute: Option<LazyTyp<'db>>,                         // [rest-names] type
  pub typ_nested_typs: EvaluatedTypNestedTyps<'db>, // { Type name [element/field] }
  pub typ_namespace: TypNamespace<'db>,             // { namespace {} } members and getters

  pub typ_constraints: BTreeMap<String, TypConstraint>, // { constraints {} }
}

impl<'db> EvaluatedTyp<'db> {
  pub fn into_element_typ(self, db: &'db dyn Database, name: String) -> Typ<'db> {
    let meta = TypElementMeta {
      labels: self.typ_element_labels(),
      settings: self.typ_element_settings(),
      setting_key_to_field: self.typ_setting_key_to_field(),
      unique_settings: self.typ_unique_settings(),
      arg_fields: self.typ_element_arg_fields(),
      attributes: self.typ_element_attributes(),
    };
    ElementTyp::new(
      db,
      name,
      self.vtable,
      Some(ObjTyp::new(db).into()),
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
      self.typ_constraints,
      meta,
    )
    .into()
  }

  fn typ_element_labels(&self) -> Vec<TypElementLabel> {
    let mut labeled: Vec<_> = self
      .typ_field_declarations
      .iter()
      .filter(|&(_name, field)| field.is_label_or_alias())
      .map(|(name, field)| TypElementLabel {
        field_name: name.clone(),
        is_alias: field.is_label_alias(),
        unique: field.is_unique(),
      })
      .collect();
    labeled
      .sort_by(|this_element, that_element| this_element.field_name.cmp(&that_element.field_name));
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
              has_csv: field.has_csv(),
              is_text: field.is_text(),
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
      .filter(|(_, field)| field.is_attribute())
      .map(|(key, field)| (key.clone(), field.typ))
      .collect();
    AttributeSchema {
      named,
      rest: self.typ_rest_attribute,
    }
  }

  fn typ_unique_settings(&self) -> BTreeSet<String> {
    self
      .typ_field_declarations
      .iter()
      .filter(|(_, field)| field.is_setting() && field.is_unique())
      .map(|(key, field)| field.setting_rename.clone().unwrap_or_else(|| key.clone()))
      .collect()
  }

  fn typ_element_settings(&self) -> BTreeMap<String, LazyTyp<'db>> {
    self
      .typ_field_declarations
      .iter()
      .filter(|(_, field)| field.is_setting())
      .map(|(key, field)| {
        let setting_rename = field.setting_rename.as_deref().unwrap_or(key.as_str());
        (setting_rename.to_string(), field.typ)
      })
      .collect()
  }

  /// Builds the external-key -> internal-field-name map for renamed settings
  fn typ_setting_key_to_field(&self) -> BTreeMap<String, String> {
    self
      .typ_field_declarations
      .iter()
      .filter(|(_, field)| field.is_setting())
      .filter_map(|(field_name, field)| {
        let external_key = field.setting_rename.as_deref()?;
        Some((external_key.to_string(), field_name.clone()))
      })
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
pub(in crate::db::pretype_evaluate_typ) struct EvaluatedTypField<'db> {
  pub typ: LazyTyp<'db>,
  pub arg_position: Option<ArgPosition>,
  pub flags: FieldAnnotationFlags,
  /// Setting key rename: only meaningful when `flags` contains `SETTING`.
  /// `None` = use field name as key, `Some(key)` = use `key` instead
  pub setting_rename: Option<String>,
  pub node: RedNode,
}

impl EvaluatedTypField<'_> {
  pub fn span(&self) -> (usize, usize) {
    (self.node.start_offset(), self.node.end_offset())
  }

  pub fn is_attribute(&self) -> bool {
    self.flags.contains(FieldAnnotationFlags::ATTRIBUTE)
  }

  pub fn is_label(&self) -> bool {
    self.flags.contains(FieldAnnotationFlags::LABEL)
  }

  pub fn is_label_alias(&self) -> bool {
    self.flags.contains(FieldAnnotationFlags::LABEL_ALIAS)
  }

  /// Returns true when the field carries either `[label]` or `[label alias]`
  pub fn is_label_or_alias(&self) -> bool {
    self
      .flags
      .intersects(FieldAnnotationFlags::LABEL | FieldAnnotationFlags::LABEL_ALIAS)
  }

  pub fn is_unique(&self) -> bool {
    self.flags.contains(FieldAnnotationFlags::UNIQUE)
  }

  pub fn is_setting(&self) -> bool {
    self.flags.contains(FieldAnnotationFlags::SETTING)
  }

  pub fn has_csv(&self) -> bool {
    self.flags.contains(FieldAnnotationFlags::CSV)
  }

  pub fn is_text(&self) -> bool {
    self.flags.contains(FieldAnnotationFlags::TEXT)
  }

  pub fn is_arg(&self) -> bool {
    self.arg_position.is_some()
  }
}

/* Partially processed type nested types */
#[derive(Default)]
pub(super) struct EvaluatedTypNestedTyps<'db> {
  pub field_nested_typs: BTreeMap<String, LazyTyp<'db>>, // { Type name [field] {} }
  pub element_nested_typs: BTreeMap<String, LazyTyp<'db>>, // { Type name [element] {} }
}

/* Partially processed type constraint */
pub(in crate::db::pretype_evaluate_typ) struct EvaluatedConstraintEntry {
  pub name: String,
  pub closure_syntax: Option<RedNode>,
  pub node: RedNode,
}

impl EvaluatedConstraintEntry {
  pub fn span(&self) -> (usize, usize) {
    (self.node.start_offset(), self.node.end_offset())
  }
}
