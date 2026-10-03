pub(super) use super::typ_element::types::{EvaluatedConstraintEntry, EvaluatedTypField};

use bitflags::bitflags;

bitflags! {
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
  pub struct FieldAnnotationFlags: u16 {
    const CSV       = 1 << 0;
    /// `[attribute]`: controls how this field appears in element instance syntax, not in the type body
    /// - On a `[field]` type: the field is written as a bare keyword inline (e.g. `pk` in `id integer pk`)
    /// - On an `[element]` type: the field is written as `key: value` in the element body
    /// May coexist with `[setting]` if the field can appear in either position
    const ATTRIBUTE           = 1 << 1;
    /// `[label]`: plain label (mutually exclusive with `LABEL_ALIAS`)
    const LABEL               = 1 << 2;
    /// `[label alias]`: alias label (mutually exclusive with `LABEL`)
    const LABEL_ALIAS         = 1 << 3;
    /// `[unique]`: field value must be unique among all instances of this type (role-dependent enforcement)
    const UNIQUE              = 1 << 4;
    /// `[setting]` or `[setting: key]`
    const SETTING             = 1 << 5;
    /// `[text]`: bare identifiers and DqStrings captured as plain text without symbol resolution
    const TEXT                = 1 << 6;
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
