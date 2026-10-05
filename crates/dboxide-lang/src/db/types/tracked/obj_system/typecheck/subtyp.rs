use std::collections::BTreeMap;

use salsa::Database;

use crate::db::types::interned::lazy_typ::LazyTyp;
use crate::db::types::interned::typ_param::TypParams;
use crate::db::types::tracked::obj_system::base::StaticTyp;
use crate::db::types::tracked::obj_system::lit::LitValue;
use crate::db::types::tracked::obj_system::{LitTyp, Typ};

/// Check if subtyp is a subtype of suptyp
pub fn is_subtyp_of<'db>(db: &'db dyn Database, subtyp: &Typ<'db>, suptyp: &Typ<'db>) -> bool {
  if subtyp == suptyp {
    return true;
  }

  // never is bottom: subtype of everything
  if matches!(subtyp, Typ::Never(_)) {
    return true;
  }

  // nothing is a subtype of never
  if matches!(suptyp, Typ::Never(_)) {
    return false;
  }

  // Sum elimination: A | B <: C iff A <: C and B <: C
  if let Typ::Sum(sub_sum) = subtyp {
    return sub_sum
      .members(db)
      .iter()
      .all(|member| is_subtyp_of(db, member, suptyp));
  }

  // After sum elimination, subtyp is never a sum

  // Phase 1: constructor compatibility (ignoring type arguments)
  if !are_constructors_compatible(db, subtyp, suptyp) {
    return false;
  }

  // Phase 2: type argument variance check
  are_typ_args_compatible(db, subtyp, suptyp)
}

/// Phase 1: check if the type constructors are compatible, ignoring type arguments
fn are_constructors_compatible<'db>(
  db: &'db dyn Database,
  subtyp: &Typ<'db>,
  suptyp: &Typ<'db>,
) -> bool {
  match suptyp {
    Typ::Any(_) | Typ::TypTyp(_) => true,
    Typ::Never(_) => false,

    Typ::Sum(sum) => sum
      .members(db)
      .iter()
      .any(|member| is_subtyp_of(db, subtyp, member)),

    Typ::String(_) => {
      matches!(
        subtyp,
        Typ::String(_) | Typ::DateTime(_) | Typ::Date(_) | Typ::Time(_) | Typ::OString(_)
      ) || matches!(subtyp, Typ::Lit(lit) if is_lit_base_match(db, lit, suptyp))
    }

    Typ::DateTime(_) => matches!(subtyp, Typ::DateTime(_)),
    Typ::Date(_) => matches!(subtyp, Typ::Date(_)),
    Typ::Time(_) => matches!(subtyp, Typ::Time(_)),

    Typ::Integer(_) => {
      matches!(subtyp, Typ::Integer(_))
        || matches!(subtyp, Typ::Lit(lit) if is_lit_base_match(db, lit, suptyp))
    }

    Typ::Float(_) => {
      matches!(subtyp, Typ::Float(_))
        || matches!(subtyp, Typ::Lit(lit) if is_lit_base_match(db, lit, suptyp))
    }

    Typ::Bool(_) => {
      matches!(subtyp, Typ::Bool(_))
        || matches!(subtyp, Typ::Lit(lit) if is_lit_base_match(db, lit, suptyp))
    }

    Typ::Null(_) => matches!(subtyp, Typ::Null(_)),
    Typ::Color(_) => matches!(subtyp, Typ::Color(_)),
    Typ::OString(_) => matches!(subtyp, Typ::OString(_)),
    Typ::Lit(_) => false,

    Typ::List(_) => matches!(subtyp, Typ::List(_)),

    Typ::Map(sup_map) => match subtyp {
      Typ::Map(_) => true,
      Typ::Product(product) => match sup_map.value(db).and_then(|v| v.resolve(db)) {
        None => true,
        Some(value_typ) => product.fields(db).values().all(|field| {
          field
            .resolve(db)
            .is_some_and(|field_typ| is_subtyp_of(db, &field_typ, &value_typ))
        }),
      },
      Typ::Element(element) => match sup_map.value(db).and_then(|v| v.resolve(db)) {
        None => true,
        Some(value_typ) => element.get_fields(db).values().all(|field| {
          field
            .resolve(db)
            .is_some_and(|field_typ| is_subtyp_of(db, &field_typ, &value_typ))
        }),
      },
      _ => false,
    },

    Typ::Func(_) => matches!(subtyp, Typ::Func(_)),

    Typ::Product(sup_product) => match subtyp {
      Typ::Product(sub_product) => {
        are_fields_compatible(db, sub_product.fields(db), sup_product.fields(db))
      }
      Typ::Element(element) => {
        are_fields_compatible(db, &element.get_fields(db), sup_product.fields(db))
      }
      _ => false,
    },

    Typ::Element(_) => match subtyp {
      Typ::Element(_) => is_nominal_subtyp_of(db, subtyp, suptyp),
      _ => false,
    },

    Typ::Declaration(_) => matches!(subtyp, Typ::Declaration(_)),
    Typ::QualifiedDeclaration(_) => matches!(subtyp, Typ::QualifiedDeclaration(_)),
    Typ::Reference(_) => matches!(subtyp, Typ::Reference(_)),
    Typ::QualifiedReference(_) => matches!(subtyp, Typ::QualifiedReference(_)),
    Typ::MetaElement(_) => matches!(subtyp, Typ::MetaElement(_)),
  }
}

/// Phase 2: check type arguments between compatible constructors
fn are_typ_args_compatible<'db>(
  db: &'db dyn Database,
  subtyp: &Typ<'db>,
  suptyp: &Typ<'db>,
) -> bool {
  match (subtyp, suptyp) {
    (Typ::List(sub_list), Typ::List(sup_list)) => match (sub_list.elem(db), sup_list.elem(db)) {
      (_, None) => true,
      (None, Some(_)) => false,
      (Some(sub_elem), Some(sup_elem)) => match (sub_elem.resolve(db), sup_elem.resolve(db)) {
        (Some(sub), Some(sup)) => is_subtyp_of(db, &sub, &sup),
        _ => false,
      },
    },

    (Typ::Map(sub_map), Typ::Map(sup_map)) => {
      let key_ok = match (sub_map.key(db), sup_map.key(db)) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(sub_key), Some(sup_key)) => match (sub_key.resolve(db), sup_key.resolve(db)) {
          (Some(sub), Some(sup)) => is_subtyp_of(db, &sub, &sup),
          _ => false,
        },
      };
      let value_ok = match (sub_map.value(db), sup_map.value(db)) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(sub_value), Some(sup_value)) => {
          match (sub_value.resolve(db), sup_value.resolve(db)) {
            (Some(sub), Some(sup)) => is_subtyp_of(db, &sub, &sup),
            _ => false,
          }
        }
      };
      key_ok && value_ok
    }

    // Contravariant params, covariant return
    (Typ::Func(sub_func), Typ::Func(sup_func)) => {
      let sub_params = sub_func.params(db);
      let sup_params = sup_func.params(db);
      if sub_params.len() != sup_params.len() {
        return false;
      }
      for (sup_param, sub_param) in sup_params.iter().zip(sub_params.iter()) {
        if !is_subtyp_of(db, sup_param, sub_param) {
          return false;
        }
      }
      is_subtyp_of(db, &sub_func.ret(db), &sup_func.ret(db))
    }

    // Declaration: invariant
    (Typ::Declaration(sub_declaration), Typ::Declaration(sup_declaration)) => {
      match (sub_declaration.param(db), sup_declaration.param(db)) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(sub_param), Some(sup_param)) => {
          match (sub_param.resolve(db), sup_param.resolve(db)) {
            (Some(sub), Some(sup)) => sub == sup,
            _ => false,
          }
        }
      }
    }
    (Typ::QualifiedDeclaration(sub_declaration), Typ::QualifiedDeclaration(sup_declaration)) => {
      match (sub_declaration.param(db), sup_declaration.param(db)) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(sub_param), Some(sup_param)) => {
          match (sub_param.resolve(db), sup_param.resolve(db)) {
            (Some(sub), Some(sup)) => sub == sup,
            _ => false,
          }
        }
      }
    }

    // Reference: covariant
    (Typ::Reference(sub_reference), Typ::Reference(sup_reference)) => {
      match (sub_reference.param(db), sup_reference.param(db)) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(sub_param), Some(sup_param)) => {
          match (sub_param.resolve(db), sup_param.resolve(db)) {
            (Some(sub), Some(sup)) => is_subtyp_of(db, &sub, &sup),
            _ => false,
          }
        }
      }
    }
    (Typ::QualifiedReference(sub_reference), Typ::QualifiedReference(sup_reference)) => {
      match (sub_reference.param(db), sup_reference.param(db)) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(sub_param), Some(sup_param)) => {
          match (sub_param.resolve(db), sup_param.resolve(db)) {
            (Some(sub), Some(sup)) => is_subtyp_of(db, &sub, &sup),
            _ => false,
          }
        }
      }
    }

    _ => true,
  }
}

/// Check if a literal typ matches a given base typ
fn is_lit_base_match<'db>(db: &'db dyn Database, lit: &LitTyp<'db>, base: &Typ<'db>) -> bool {
  match &lit.value(db) {
    LitValue::String(_) => matches!(base, Typ::String(_)),
    LitValue::Integer(_) => matches!(base, Typ::Integer(_)),
    LitValue::Float(_) => matches!(base, Typ::Float(_)),
    LitValue::Bool(_) => matches!(base, Typ::Bool(_)),
  }
}

/// Structural field compatibility (width + depth subtyping)
fn are_fields_compatible<'db>(
  db: &'db dyn Database,
  sub_fields: &BTreeMap<String, LazyTyp<'db>>,
  sup_fields: &BTreeMap<String, LazyTyp<'db>>,
) -> bool {
  sup_fields.iter().all(|(name, sup_field)| {
    let optional = sup_field
      .as_eager()
      .is_some_and(|typ| is_nullable(db, &typ));
    match sub_fields.get(name) {
      Some(sub_field) => match (sub_field.as_eager(), sup_field.as_eager()) {
        (Some(subtyp), Some(suptyp)) => is_subtyp_of(db, &subtyp, &suptyp),
        _ => false,
      },
      None => optional,
    }
  })
}

/// Walk the parent chain checking identity (with cycle detection)
fn is_nominal_subtyp_of<'db>(db: &'db dyn Database, subtyp: &Typ<'db>, suptyp: &Typ<'db>) -> bool {
  let mut visited = hashbrown::HashSet::new();
  visited.insert(*subtyp);
  let mut current = subtyp.parent_typ(db);
  while let Some(parent) = current {
    if &parent == suptyp {
      return true;
    }
    if !visited.insert(parent) {
      return false;
    }
    current = parent.parent_typ(db);
  }
  false
}

/// Check if a typ includes null
pub fn is_nullable<'db>(db: &'db dyn Database, typ: &Typ<'db>) -> bool {
  match typ {
    Typ::Null(_) => true,
    Typ::Sum(sum) => sum
      .members(db)
      .iter()
      .any(|member| matches!(member, Typ::Null(_))),
    _ => false,
  }
}

/// Validate type arguments against type parameters for arity and bound violations
pub fn validate_typ_params<'db>(
  db: &'db dyn Database,
  typ_params: Option<&TypParams<'db>>,
  args: &[LazyTyp<'db>],
) -> bool {
  let expected_arity = typ_params.map_or(0, |p| p.params(db).len());
  if expected_arity != args.len() {
    return false;
  }
  let Some(params) = typ_params else {
    return true;
  };
  let params_vec = params.params(db);
  for (param, arg) in params_vec.iter().zip(args.iter()) {
    if let Some(bound) = param.upper_bound(db)
      && let (Some(bound_typ), Some(arg_typ)) = (bound.as_eager(), arg.as_eager())
      && !is_subtyp_of(db, &arg_typ, &bound_typ)
    {
      return false;
    }
  }
  true
}
