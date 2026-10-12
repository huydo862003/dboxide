use std::collections::BTreeMap;

use salsa::{Database, tracked};

use crate::ast::RedNode;
use crate::db::types::interned::lazy_typ::LazyTyp;

use super::base::{RuntimeObj, StaticTyp};
use super::{FuncObj, Obj, Typ};

/// Policy for how a type behaves when used in an expression context
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum OnUsePolicy {
  #[default]
  Default,
  Expand,
}

/// Element meta type (metatype of all element types)
#[tracked]
pub struct ElementMetaTyp<'db> {
  pub name: String,
  pub fields: BTreeMap<String, LazyTyp<'db>>,
  pub namespace: BTreeMap<String, LazyTyp<'db>>,
  pub parent: Option<Typ<'db>>,
}

pub type MetaElementTyp<'db> = ElementMetaTyp<'db>;

impl<'db> StaticTyp<'db> for ElementMetaTyp<'db> {
  fn display_name(&self, db: &'db dyn Database) -> String {
    self.name(db).clone()
  }

  fn parent_typ(&self, db: &'db dyn Database) -> Option<Typ<'db>> {
    *self.parent(db)
  }

  fn is_typ(&self, _db: &'db dyn Database) -> bool {
    true
  }

  fn get_fields(&self, db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    self.fields(db).clone()
  }

  fn get_namespace_members(&self, db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    self.namespace(db).clone()
  }
}

impl<'db> RuntimeObj<'db> for ElementMetaTyp<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    Typ::TypTyp(super::TypTyp::new(db))
  }
}

/// Positional arg field declared on an element or field type
#[derive(Clone, PartialEq, Eq, Hash, Debug, salsa::SalsaValue)]
pub struct ArgFieldDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
  pub is_variadic: bool,
  pub has_csv: bool,
}

/// Schema for attributes on an element type.
/// `named` holds fields declared with `[attribute]`.
/// `rest` holds the catch-all type from a `[name] Type` bracket-syntax declaration: when set, any attribute key in the element body is accepted and its value is typed against `rest`.
#[derive(Clone, PartialEq, Eq, Hash, Default, salsa::SalsaValue)]
pub struct AttributeSchema<'db> {
  pub named: BTreeMap<String, LazyTyp<'db>>,
  pub rest: Option<LazyTyp<'db>>,
}

/// Nested type definitions inside a type body:
/// - `field`: `type Name [field] { ... }`: defines how a "field line" in the element body looks
/// - `element`: `type Name [element] { ... }`: defines a sub-element block (`indexes { }`, etc.)
#[derive(Clone, PartialEq, Eq, Hash, Default, salsa::SalsaValue)]
pub struct TypNestedTyps<'db> {
  pub field_nested_typs: BTreeMap<String, LazyTyp<'db>>,
  pub element_nested_typs: BTreeMap<String, LazyTyp<'db>>,
}

/// Element type
#[tracked]
pub struct ElementTyp<'db> {
  pub name: String,
  pub vtable: BTreeMap<String, FuncObj<'db>>,
  pub parent: Option<Typ<'db>>,

  pub typ_settings: TypSettings, // Type name [on use: expand, element, field]
  pub typ_field_lines: BTreeMap<String, LazyTyp<'db>>, // field-name field-type
  /// Nested field and element type definitions from the type body
  pub typ_nested_typs: TypNestedTyps<'db>,
  pub typ_namespace: BTreeMap<String, LazyTyp<'db>>,
  pub typ_constraints: BTreeMap<String, TypConstraint>,

  /* Specific fields */
  pub typ_element_labels: Vec<TypElementLabel>, // field-name field-type [label] -> declaring Element label []
  pub typ_element_settings: BTreeMap<String, LazyTyp<'db>>, // field-name field-type [setting] -> declaring Type name [field-name: field-type]
  pub typ_element_arg_fields: BTreeMap<usize, ArgFieldDefinition<'db>>, // Positional arg fields by index, from `[arg: n]` or `[arg: n..]` annotations
  pub typ_element_attributes: AttributeSchema<'db>, // field-name field-type [attribute] -> declaring field-name: field-type
}

impl<'db> ElementTyp<'db> {
  pub fn label_definitions(&self, db: &'db dyn Database) -> Vec<TypElementLabel> {
    self.typ_element_labels(db).clone()
  }

  pub fn field_definitions(&self, db: &'db dyn Database) -> Vec<TypElementFieldDefinition<'db>> {
    self
      .typ_field_lines(db)
      .iter()
      .map(|(name, typ)| TypElementFieldDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn attribute_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<TypElementAttributeDefinition<'db>> {
    self
      .typ_element_attributes(db)
      .named
      .iter()
      .map(|(name, typ)| TypElementAttributeDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn setting_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<TypElementSettingDefinition<'db>> {
    self
      .typ_element_settings(db)
      .iter()
      .map(|(name, typ)| TypElementSettingDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn field_nested_typ_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<TypNestedFieldTypDefinition<'db>> {
    self
      .typ_nested_typs(db)
      .field_nested_typs
      .iter()
      .map(|(name, typ)| TypNestedFieldTypDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn element_nested_typ_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<TypNestedElementTypDefinition<'db>> {
    self
      .typ_nested_typs(db)
      .element_nested_typs
      .iter()
      .map(|(name, typ)| TypNestedElementTypDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn nested_typ_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<ElementNestedElementDefinition<'db>> {
    self
      .typ_nested_typs(db)
      .element_nested_typs
      .iter()
      .map(|(name, typ)| ElementNestedElementDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn constraint_definitions(&self, db: &'db dyn Database) -> Vec<TypConstraint> {
    self.typ_constraints(db).values().cloned().collect()
  }

  pub fn lookup_constraint(&self, db: &'db dyn Database, name: &str) -> Option<TypConstraint> {
    self.typ_constraints(db).get(name).cloned()
  }
}

impl<'db> StaticTyp<'db> for ElementTyp<'db> {
  fn display_name(&self, db: &'db dyn Database) -> String {
    self.name(db).clone()
  }

  fn parent_typ(&self, db: &'db dyn Database) -> Option<Typ<'db>> {
    *self.parent(db)
  }

  fn is_typ(&self, _db: &'db dyn Database) -> bool {
    true
  }

  fn runtime_typ(&self, _db: &'db dyn Database) -> Option<Typ<'db>> {
    Some((*self).into())
  }

  fn get_fields(&self, db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    let mut result = self
      .parent(db)
      .map(|parent| parent.get_fields(db))
      .unwrap_or_default();
    result.extend(self.typ_field_lines(db).clone());
    result
  }

  fn get_namespace_members(&self, db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    let mut result = self
      .parent(db)
      .map(|parent| parent.get_namespace_members(db))
      .unwrap_or_default();
    result.extend(self.typ_namespace(db).clone());
    result
  }

  fn lookup_namespace_member(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    if let Some(member) = self.typ_namespace(db).get(name) {
      return Some(*member);
    }
    self
      .parent_typ(db)
      .and_then(|p| p.lookup_namespace_member(db, name))
  }

  fn runtime_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, super::FuncObj<'db>> {
    let mut result = self
      .parent_typ(db)
      .map(|parent| parent.runtime_vtable(db))
      .unwrap_or_default();
    result.extend(self.vtable(db).clone());
    result
  }

  fn static_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, Typ<'db>> {
    let mut result = self
      .parent_typ(db)
      .map(|parent| parent.static_vtable(db))
      .unwrap_or_default();
    for (name, func_obj) in self.vtable(db) {
      result.insert(name.clone(), func_obj.get_typ(db));
    }
    result
  }
}

impl<'db> RuntimeObj<'db> for ElementTyp<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    Typ::TypTyp(super::TypTyp::new(db))
  }

  fn source_path(&self, db: &'db dyn Database) -> String {
    self.display_name(db)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct TypSettings {
  pub on_use: OnUsePolicy,
  pub is_element_typ: bool,
  pub is_field_typ: bool,
}

/// Label definition representing a declared name or alias slot
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypElementLabel {
  pub field_name: String,
  pub is_alias: bool,
  pub unique: bool,
  pub exposes_namespace: bool,
}

/// Field definition representing a declared field on an element
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypElementFieldDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Attribute definition representing a declared attribute (attr: type)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypElementAttributeDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Setting definition representing a declared setting ([setting])
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypElementSettingDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Field nested type definition representing a nested field type ([field])
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypNestedFieldTypDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Element nested type definition representing a nested element type ([element])
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypNestedElementTypDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Nested element definition representing a nested element type ([element])
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ElementNestedElementDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Constraint definition representing a validation rule inside constraints block
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypConstraint {
  pub name: String,
  pub closure_syntax: RedNode,
}

/// Runtime instance of an element type
#[tracked]
pub struct ElementObj<'db> {
  #[returns(copy)]
  pub element_typ: Typ<'db>,
  #[returns(ref)]
  pub fields: BTreeMap<String, Obj<'db>>,
}

impl<'db> RuntimeObj<'db> for ElementObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    self.element_typ(db)
  }

  fn get_owned_field(&self, db: &'db dyn Database, key: &str) -> Option<super::Obj<'db>> {
    self.fields(db).get(key).copied()
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    format!("{}(...)", self.element_typ(db).display_name(db))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use salsa::tracked;

  use super::*;
  use crate::db::DboxideDatabase;
  use crate::db::types::FuncSignature;
  use crate::db::types::tracked::obj_system::{
    FuncObj, FuncTyp, IntegerObj, IntegerTyp, StringTyp,
  };

  #[tracked]
  fn run_element_vtable_test(db: &dyn Database) {
    let string_typ = Typ::String(StringTyp::new(db));
    let integer_typ = Typ::Integer(IntegerTyp::new(db));

    let signature_greet = FuncSignature::new(db, vec![], LazyTyp::eager(string_typ));
    let greet_func = FuncObj::new(
      db,
      "greet".to_string(),
      FuncTyp::new(db, signature_greet),
      None,
      None,
    );

    let mut parent_vtable = BTreeMap::new();
    parent_vtable.insert("greet".to_string(), greet_func);

    let parent_element_typ = ElementTyp::new(
      db,
      "BaseElement".to_string(),
      parent_vtable,
      None,
      TypSettings::default(),
      BTreeMap::new(),
      TypNestedTyps::default(),
      BTreeMap::new(),
      BTreeMap::new(),
      vec![],
      BTreeMap::new(),
      BTreeMap::new(),
      AttributeSchema::default(),
    );

    let signature_calc = FuncSignature::new(
      db,
      vec![LazyTyp::eager(integer_typ)],
      LazyTyp::eager(integer_typ),
    );
    let calc_func = FuncObj::new(
      db,
      "calc".to_string(),
      FuncTyp::new(db, signature_calc),
      None,
      None,
    );

    let mut child_vtable = BTreeMap::new();
    child_vtable.insert("calc".to_string(), calc_func);

    let mut child_fields = BTreeMap::new();
    child_fields.insert("id".to_string(), LazyTyp::eager(integer_typ));

    let child_element_typ = ElementTyp::new(
      db,
      "ChildElement".to_string(),
      child_vtable,
      Some(Typ::Element(parent_element_typ)),
      TypSettings::default(),
      child_fields,
      TypNestedTyps::default(),
      BTreeMap::new(),
      BTreeMap::new(),
      vec![],
      BTreeMap::new(),
      BTreeMap::new(),
      AttributeSchema::default(),
    );

    // Static lookup tests
    assert!(child_element_typ.lookup_field_typ(db, "id").is_some());
    assert!(
      child_element_typ
        .lookup_field_typ(db, "id")
        .unwrap()
        .as_eager()
        == Some(integer_typ)
    );

    let calc_static = child_element_typ.lookup_field_typ(db, "calc");
    assert!(calc_static.is_some());
    assert!(matches!(
      calc_static.unwrap().as_eager(),
      Some(Typ::Func(_))
    ));

    let greet_static = child_element_typ.lookup_field_typ(db, "greet");
    assert!(greet_static.is_some());
    assert!(matches!(
      greet_static.unwrap().as_eager(),
      Some(Typ::Func(_))
    ));

    assert!(
      child_element_typ
        .lookup_field_typ(db, "nonexistent")
        .is_none()
    );

    // Runtime object lookup tests
    let mut obj_fields = BTreeMap::new();
    let integer_val = super::super::Obj::Integer(IntegerObj::new(db, 42));
    obj_fields.insert("id".to_string(), integer_val);

    let element_obj = ElementObj::new(db, Typ::Element(child_element_typ), obj_fields);

    assert!(element_obj.get_owned_field(db, "id") == Some(integer_val));
    assert!(element_obj.lookup_field(db, "id") == Some(integer_val));

    let calc_method = element_obj.lookup_method(db, "calc");
    assert!(calc_method == Some(calc_func));

    let greet_method = element_obj.lookup_method(db, "greet");
    assert!(greet_method == Some(greet_func));

    let calc_via_field = element_obj.lookup_field(db, "calc");
    assert!(calc_via_field == Some(super::super::Obj::Func(calc_func)));

    let greet_via_field = element_obj.lookup_field(db, "greet");
    assert!(greet_via_field == Some(super::super::Obj::Func(greet_func)));

    assert!(element_obj.lookup_field(db, "missing").is_none());
  }

  #[test]
  fn test_element_static_and_runtime_vtable() {
    let db = DboxideDatabase::default();
    run_element_vtable_test(&db);
  }
}
