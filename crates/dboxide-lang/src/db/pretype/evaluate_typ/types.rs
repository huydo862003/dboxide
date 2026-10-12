pub(super) use super::typ_element::types::{EvaluatedConstraintEntry, EvaluatedTypField};

use bitflags::bitflags;

bitflags! {
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
  pub struct FieldAnnotationFlags: u8 {
    const CSV       = 1 << 0;
    /// `[attribute]`: controls how this field appears in element **instance** syntax, not in the type body
    /// - On a `[field]` type: the field is written as a bare keyword inline (e.g. `pk` in `id integer pk`)
    /// - On an `[element]` type: the field is written as `key: value` in the element body
    /// May coexist with `[setting]` if the field can appear in either position
    const ATTRIBUTE = 1 << 1;
  }
}

bitflags! {
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
  pub struct LabelFlags: u8 {
    /// `[label]`: plain label (mutually exclusive with `ALIAS`)
    const LABEL             = 1 << 0;
    /// `[label alias]`: alias label (mutually exclusive with `LABEL`)
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
