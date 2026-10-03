use std::collections::{BTreeMap, BTreeSet};

use salsa::{Database, SalsaValue, tracked};

use crate::TypTyp;
use crate::ast::RedNode;
use crate::db::types::interned::lazy_typ::LazyTyp;

use super::base::{RuntimeObj, StaticTyp};
use super::{FuncObj, Obj, Typ};

/// Policy for how a type behaves when used in an expression context
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum OnUsePolicy {
  #[default]
  Default,
  ExpandReferences,
  /// Anonymous instances of this type follow their referenced names into scope during wildcard use
  FollowRefs,
}

/// Element meta type (metatype of all element types)
#[tracked]
pub struct ElementMetaTyp<'db> {
  pub name: String,
  pub fields: BTreeMap<String, LazyTyp<'db>>,
  pub namespace: BTreeMap<String, LazyTyp<'db>>,
  pub parent: Option<Typ<'db>>,
}

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
    Typ::TypTyp(TypTyp::new(db))
  }
}

/// Positional arg field declared on an element or field type
#[derive(Clone, PartialEq, Eq, Hash, Debug, SalsaValue)]
pub struct ArgFieldDefinition<'db> {
  pub name: String,
  pub typ: LazyTyp<'db>,
  pub is_variadic: bool,
  pub has_csv: bool,
  /// When true, bare identifiers and DqStrings in this slot are captured as plain text
  /// without symbol resolution (declared with `[text]` on the field)
  pub is_text: bool,
}

/// Schema for attributes on an element type
#[derive(Clone, PartialEq, Eq, Hash, Default, SalsaValue)]
pub struct AttributeSchema<'db> {
  pub named: BTreeMap<String, LazyTyp<'db>>,
  pub rest: Option<LazyTyp<'db>>,
}

/// Nested type definitions inside a type body:
/// - `field`: `type Name [field] { ... }`: defines how a "field line" in the element body looks
/// - `element`: `type Name [element] { ... }`: defines a sub-element block (`indexes { }`, etc.)
#[derive(Clone, PartialEq, Eq, Hash, Default, SalsaValue)]
pub struct TypNestedTyps<'db> {
  pub field_nested_typs: BTreeMap<String, LazyTyp<'db>>,
  pub element_nested_typs: BTreeMap<String, LazyTyp<'db>>,
}

/// Namespace of an element type: field re-exports and computed getters
#[derive(Clone, PartialEq, Eq, Hash, Default, SalsaValue)]
pub struct TypNamespace<'db> {
  /// Accessible via `.name`, return type of field re-exports and getters
  pub members: BTreeMap<String, LazyTyp<'db>>,
  /// `get name(): Type {}` getter implementations inside `namespace {}`
  pub getters: BTreeMap<String, FuncObj<'db>>,
  /// Field name declared with `name [expand]` for by-name expansion into namespace
  pub expanded_field: Option<String>,
  /// Item type of the expanded list field (`T` from `T[]`)
  pub expanded_item_typ: Option<LazyTyp<'db>>,
}

/// Element-specific metadata derived from field annotations
#[derive(Clone, PartialEq, Eq, Hash, Default, SalsaValue)]
pub struct TypElementMeta<'db> {
  pub labels: Vec<TypElementLabel>,
  pub settings: BTreeMap<String, LazyTyp<'db>>,
  /// Maps external setting key -> internal field name for renamed settings
  pub setting_key_to_field: BTreeMap<String, String>,
  /// Setting keys that must not appear more than once on an element instance
  pub unique_settings: BTreeSet<String>,
  pub arg_fields: BTreeMap<usize, ArgFieldDefinition<'db>>,
  pub attributes: AttributeSchema<'db>,
}

/// Element type
#[tracked]
pub struct ElementTyp<'db> {
  pub name: String,
  pub vtable: BTreeMap<String, FuncObj<'db>>,
  pub parent: Option<Typ<'db>>,

  pub typ_settings: TypSettings, // Type name [use: expand namespaces, element, field]
  pub typ_field_lines: BTreeMap<String, LazyTyp<'db>>, // field-name field-type
  /// Nested field and element type definitions from the type body
  pub typ_nested_typs: TypNestedTyps<'db>,
  pub typ_namespace: TypNamespace<'db>,
  pub typ_constraints: BTreeMap<String, TypConstraint>,
  pub typ_element_meta: TypElementMeta<'db>,
}

impl<'db> ElementTyp<'db> {
  pub fn typ_element_labels(&self, db: &'db dyn Database) -> &Vec<TypElementLabel> {
    &self.typ_element_meta(db).labels
  }

  pub fn typ_element_settings(&self, db: &'db dyn Database) -> &BTreeMap<String, LazyTyp<'db>> {
    &self.typ_element_meta(db).settings
  }

  /// Resolves a user-written setting key to the internal field name
  pub fn setting_field_name(&self, db: &'db dyn Database, setting_key: &str) -> String {
    self
      .typ_element_meta(db)
      .setting_key_to_field
      .get(setting_key)
      .cloned()
      .unwrap_or_else(|| setting_key.to_string())
  }

  pub fn typ_unique_settings(&self, db: &'db dyn Database) -> &BTreeSet<String> {
    &self.typ_element_meta(db).unique_settings
  }

  pub fn typ_element_arg_fields(
    &self,
    db: &'db dyn Database,
  ) -> &BTreeMap<usize, ArgFieldDefinition<'db>> {
    &self.typ_element_meta(db).arg_fields
  }

  pub fn typ_element_attributes(&self, db: &'db dyn Database) -> &AttributeSchema<'db> {
    &self.typ_element_meta(db).attributes
  }

  pub fn label_definitions(&self, db: &'db dyn Database) -> Vec<TypElementLabel> {
    self.typ_element_meta(db).labels.clone()
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

  /// Finds the field name that holds instances of a nested type
  /// Matches both `columns Column[]` (list) and `note Note` (singular)
  pub fn collection_field_name(
    &self,
    db: &'db dyn Database,
    nested_type_name: &str,
  ) -> Option<String> {
    for field in self.field_definitions(db) {
      let Some(resolved) = field.typ.resolve(db) else {
        continue;
      };
      match resolved {
        Typ::List(list_typ) => {
          if let Some(element_lazy) = list_typ.element(db)
            && let Some(Typ::Element(inner)) = element_lazy.resolve(db)
            && inner.name(db).eq_ignore_ascii_case(nested_type_name)
          {
            return Some(field.name);
          }
        }
        Typ::Element(inner) if inner.name(db).eq_ignore_ascii_case(nested_type_name) => {
          return Some(field.name);
        }
        _ => {}
      }
    }
    None
  }

  /// Returns the nested [field] type for this element, if any
  pub fn field_line_typ(&self, db: &'db dyn Database) -> Option<ElementTyp<'db>> {
    self
      .typ_nested_typs(db)
      .field_nested_typs
      .values()
      .find_map(|lazy_typ| match lazy_typ.resolve(db) {
        Some(Typ::Element(element_typ)) => Some(element_typ),
        _ => None,
      })
  }

  pub fn attribute_definitions(
    &self,
    db: &'db dyn Database,
  ) -> Vec<TypElementAttributeDefinition<'db>> {
    self
      .typ_element_meta(db)
      .attributes
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
      .typ_element_meta(db)
      .settings
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
    result.extend(self.typ_namespace(db).members.clone());
    result
  }

  fn get_namespace_member(&self, db: &'db dyn Database, name: &str) -> Option<LazyTyp<'db>> {
    let namespace = self.typ_namespace(db);
    if let Some(member) = namespace.members.get(name) {
      return Some(*member);
    }
    // Nested types are accessible via `.` on the parent type
    let nested = self.typ_nested_typs(db);
    if let Some(lazy) = nested.field_nested_typs.get(name) {
      return Some(*lazy);
    }
    if let Some(lazy) = nested.element_nested_typs.get(name) {
      return Some(*lazy);
    }
    if let parent_result @ Some(_) = self
      .parent_typ(db)
      .and_then(|parent| parent.get_namespace_member(db, name))
    {
      return parent_result;
    }
    // Expanded field fallback: [expand] means any unmatched name returns the item type
    if let Some(item_typ) = namespace.expanded_item_typ {
      return Some(item_typ);
    }
    None
  }

  fn runtime_vtable(&self, db: &'db dyn Database) -> BTreeMap<String, FuncObj<'db>> {
    let mut result = self
      .parent_typ(db)
      .map(|parent| parent.runtime_vtable(db))
      .unwrap_or_default();
    result.extend(self.vtable(db).clone());
    result
  }

  fn lookup_runtime_op(&self, db: &'db dyn Database, op: &str) -> Option<FuncObj<'db>> {
    self
      .vtable(db)
      .get(op)
      .copied()
      .or_else(|| self.parent_typ(db)?.lookup_runtime_op(db, op))
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
    Typ::TypTyp(TypTyp::new(db))
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

  fn get_owned_field(&self, db: &'db dyn Database, key: &str) -> Option<Obj<'db>> {
    self.fields(db).get(key).copied()
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    format!("{}(...)", self.element_typ(db).display_name(db))
  }

  fn lookup_namespace_member(&self, db: &'db dyn Database, name: &str) -> Option<Obj<'db>> {
    let Typ::Element(element_typ) = self.element_typ(db) else {
      return None;
    };
    let namespace = element_typ.typ_namespace(db);

    // Check getters first
    if let Some(getter) = namespace.getters.get(name) {
      return Some(Obj::Func(*getter));
    }

    // Expanded field fallback: search the list for an item whose "name" field matches
    if let Some(ref field_name) = namespace.expanded_field {
      if let Some(Obj::List(list)) = self.fields(db).get(field_name) {
        for item in list.items(db) {
          if let Obj::Element(element) = item
            && let Some(Obj::String(string)) = element.fields(db).get("name")
            && string.value(db) == name
          {
            return Some(*item);
          }
        }
      }
      // Item not found in expanded list: caller handles the diagnostic
      return None;
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use salsa::tracked;

  use super::*;
  use crate::ObjTyp;
  use crate::db::DboxideDatabase;
  use crate::db::types::FuncSignature;
  use crate::db::types::tracked::obj_system::{
    FuncObj, FuncTyp, IntegerObj, IntegerTyp, StringTyp,
  };

  #[test]
  fn test_element_static_and_runtime_vtable() {
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
        Some(ObjTyp::new(db).into()),
        TypSettings::default(),
        BTreeMap::new(),
        TypNestedTyps::default(),
        TypNamespace::default(),
        BTreeMap::new(),
        TypElementMeta::default(),
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
        TypNamespace::default(),
        BTreeMap::new(),
        TypElementMeta::default(),
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
      let integer_value = Obj::Integer(IntegerObj::new(db, 42));
      obj_fields.insert("id".to_string(), integer_value);

      let element_obj = ElementObj::new(db, Typ::Element(child_element_typ), obj_fields);

      assert!(element_obj.get_owned_field(db, "id") == Some(integer_value));
      assert!(element_obj.lookup_field(db, "id") == Some(integer_value));

      let calc_method = element_obj.lookup_method(db, "calc");
      assert!(calc_method == Some(calc_func));

      let greet_method = element_obj.lookup_method(db, "greet");
      assert!(greet_method == Some(greet_func));

      let calc_via_field = element_obj.lookup_field(db, "calc");
      assert!(calc_via_field == Some(Obj::Func(calc_func)));

      let greet_via_field = element_obj.lookup_field(db, "greet");
      assert!(greet_via_field == Some(Obj::Func(greet_func)));

      assert!(element_obj.lookup_field(db, "missing").is_none());
    }

    let db = DboxideDatabase::default();
    run_element_vtable_test(&db);
  }

  #[test]
  fn test_element_obj_lookup_namespace_member_returns_getter_func() {
    #[tracked]
    fn run(db: &dyn Database) {
      let string_typ = Typ::String(StringTyp::new(db));
      let signature = FuncSignature::new(db, vec![], LazyTyp::eager(string_typ));
      let getter_func = FuncObj::new(
        db,
        "label".to_string(),
        FuncTyp::new(db, signature),
        None,
        None,
      );
      let mut getters = BTreeMap::new();
      getters.insert("label".to_string(), getter_func);
      let namespace = TypNamespace {
        members: BTreeMap::new(),
        getters,
        expanded_field: None,
        expanded_item_typ: None,
      };
      let element_typ = ElementTyp::new(
        db,
        "MyTyp".to_string(),
        BTreeMap::new(),
        None,
        TypSettings::default(),
        BTreeMap::new(),
        TypNestedTyps::default(),
        namespace,
        BTreeMap::new(),
        TypElementMeta::default(),
      );
      let element_obj = ElementObj::new(db, Typ::Element(element_typ), BTreeMap::new());

      let result = element_obj.lookup_namespace_member(db, "label");
      assert!(result == Some(Obj::Func(getter_func)));
      assert!(
        element_obj
          .lookup_namespace_member(db, "nonexistent")
          .is_none()
      );
    }

    let db = DboxideDatabase::default();
    run(&db);
  }
}
