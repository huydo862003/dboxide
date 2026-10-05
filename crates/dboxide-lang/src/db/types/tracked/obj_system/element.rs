use std::collections::BTreeMap;

use salsa::{Database, tracked};

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

/// Element type
#[tracked]
pub struct ElementTyp<'db> {
  pub name: String,
  pub fields: BTreeMap<String, LazyTyp<'db>>,
  pub labels: Vec<LabelDefinition>,
  pub attributes: BTreeMap<String, LazyTyp<'db>>,
  pub settings: BTreeMap<String, LazyTyp<'db>>,
  pub field_subtypes: BTreeMap<String, LazyTyp<'db>>,
  pub element_subtypes: BTreeMap<String, LazyTyp<'db>>,
  pub namespace: BTreeMap<String, LazyTyp<'db>>,
  pub vtable: BTreeMap<String, FuncObj<'db>>,
  pub constraints: BTreeMap<String, ConstraintDefinition>,
  #[returns(copy)]
  pub on_use: OnUsePolicy,
  pub parent: Option<Typ<'db>>,
}

impl<'db> ElementTyp<'db> {
  pub fn label_definitions(&self, db: &'db dyn Database) -> Vec<LabelDefinition> {
    self.labels(db).clone()
  }

  pub fn field_definitions(&self, db: &'db dyn Database) -> Vec<FieldDefinition<'db>> {
    self
      .fields(db)
      .iter()
      .map(|(name, typ)| FieldDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn attribute_definitions(&self, db: &'db dyn Database) -> Vec<AttributeDefinition<'db>> {
    self
      .attributes(db)
      .iter()
      .map(|(name, typ)| AttributeDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn setting_definitions(&self, db: &'db dyn Database) -> Vec<SettingDefinition<'db>> {
    self
      .settings(db)
      .iter()
      .map(|(name, typ)| SettingDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn field_subtype_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<FieldSubTypeDefinition<'db>> {
    self
      .field_subtypes(db)
      .iter()
      .map(|(name, typ)| FieldSubTypeDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn field_sub_type_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<FieldSubTypeDefinition<'db>> {
    self.field_subtype_definitions(db)
  }

  pub fn element_subtype_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<ElementSubTypeDefinition<'db>> {
    self
      .element_subtypes(db)
      .iter()
      .map(|(name, typ)| ElementSubTypeDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn element_sub_type_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<ElementSubTypeDefinition<'db>> {
    self.element_subtype_definitions(db)
  }

  pub fn sub_element_definitions(&self, db: &'db dyn Database) -> Vec<SubElementDefinition<'db>> {
    self
      .element_subtypes(db)
      .iter()
      .map(|(name, typ)| SubElementDefinition {
        name: name.clone(),
        typ: *typ,
      })
      .collect()
  }

  pub fn constraint_definitions(&self, db: &'db dyn Database) -> Vec<ConstraintDefinition> {
    self.constraints(db).values().cloned().collect()
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
      .map(|p| p.get_fields(db))
      .unwrap_or_default();
    result.extend(self.fields(db).clone());
    result
  }

  fn get_namespace_members(&self, db: &'db dyn Database) -> BTreeMap<String, LazyTyp<'db>> {
    let mut result = self
      .parent(db)
      .map(|p| p.get_namespace_members(db))
      .unwrap_or_default();
    result.extend(self.namespace(db).clone());
    result
  }

  fn lookup_namespace_member(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    if let Some(member) = self.namespace(db).get(name) {
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

/// Label definition representing a declared name or alias slot
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LabelDefinition {
  pub field_name: String,
  pub is_alias: bool,
  pub unique: bool,
  pub exposes_namespace: bool,
}

/// Field definition representing a declared field on an element
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FieldDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Attribute definition representing a declared attribute (@attr or attr: type)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AttributeDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Setting definition representing a declared setting ([setting])
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SettingDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Field sub-type definition representing a nested field type ([field])
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FieldSubTypeDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Sub-element definition representing a nested element type ([element])
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SubElementDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Element sub-type definition representing a nested element type ([element])
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ElementSubTypeDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
}

/// Constraint definition representing a validation rule inside constraints block
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConstraintDefinition {
  pub name: String,
  pub closure_syntax: crate::ast::RedNode,
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

    let signature_greet = FuncSignature::new(db, vec![], string_typ);
    let greet_func = FuncObj::new(
      db,
      "greet".to_string(),
      FuncTyp::new(db, signature_greet),
      None,
    );

    let mut parent_vtable = BTreeMap::new();
    parent_vtable.insert("greet".to_string(), greet_func);

    let parent_element_typ = ElementTyp::new(
      db,
      "BaseElement".to_string(),
      BTreeMap::new(),
      vec![],
      BTreeMap::new(),
      BTreeMap::new(),
      BTreeMap::new(),
      BTreeMap::new(),
      BTreeMap::new(),
      parent_vtable,
      BTreeMap::new(),
      OnUsePolicy::Default,
      None,
    );

    let signature_calc = FuncSignature::new(db, vec![integer_typ], integer_typ);
    let calc_func = FuncObj::new(
      db,
      "calc".to_string(),
      FuncTyp::new(db, signature_calc),
      None,
    );

    let mut child_vtable = BTreeMap::new();
    child_vtable.insert("calc".to_string(), calc_func);

    let mut child_fields = BTreeMap::new();
    child_fields.insert("id".to_string(), LazyTyp::eager(integer_typ));

    let child_element_typ = ElementTyp::new(
      db,
      "ChildElement".to_string(),
      child_fields,
      vec![],
      BTreeMap::new(),
      BTreeMap::new(),
      BTreeMap::new(),
      BTreeMap::new(),
      BTreeMap::new(),
      child_vtable,
      BTreeMap::new(),
      OnUsePolicy::Default,
      Some(Typ::Element(parent_element_typ)),
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
