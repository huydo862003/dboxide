//! Salsa database and incremental queries
//!
//! Pipeline phases:
//!
//! 1. parse:            `File` -> AST, use specifiers
//! 2. pretype_analyze:  resolve, scope, namespace (types and schemas only)
//! 3. pretype_evaluate_typ:     `Symbol` -> `EvaluatedTyp`; includes pretype expression eval
//! 4. posttype_analyze: resolve, scope, namespace (all symbols); typecheck, validate, declare
//! 5. posttype_evaluate_element: `File` -> `Vec<ElementObj>`; includes posttype expression eval
//!
//! Import resolution is host-driven:
//! 1. Host creates `File` inputs with content
//! 2. `parse_file` produces the AST (cached by salsa)
//! 3. `get_use_specifiers` extracts unresolved specifiers from the AST
//! 4. Host resolves each specifier and sets `File.uses`
//! 5. `pretype_get_scope_members` reads `File.uses` to merge imported symbols
//!
//! The host owns specifier resolution (relative paths, import maps, etc).
//! The language never touches the filesystem.

#[cfg(test)]
use std::sync::{Arc, Mutex};

#[cfg(test)]
pub(crate) mod test_utils;

use salsa::{Database, Storage, db};

// TIL: #[path] lets folders use numeric prefixes for ordering while module names stay clean
#[path = "1_parse/mod.rs"]
mod parse;
#[path = "4_posttype_analyze/mod.rs"]
mod posttype_analyze;
#[path = "5_posttype_evaluate_element/mod.rs"]
mod posttype_evaluate_element;
#[path = "2_pretype_analyze/mod.rs"]
mod pretype_analyze;
#[path = "3_pretype_evaluate_typ/mod.rs"]
mod pretype_evaluate_typ;
mod types;

pub use parse::*;
pub use posttype_analyze::validate_file_elements;
pub use posttype_analyze::{
  get_node_expected_typ, posttype_get_file_symbols, posttype_get_scope_members,
  posttype_get_symbol_namespace, posttype_resolve_referee,
};
pub use posttype_evaluate_element::evaluate_file;
pub use pretype_analyze::{
  get_declaration_typ, get_node_static_scope, pretype_get_file_symbols, pretype_get_scope_members,
  pretype_get_symbol_namespace, pretype_resolve_referee,
};
pub use pretype_evaluate_typ::pretype_evaluate_typ_symbol;
pub use types::*;

#[db]
#[derive(Clone)]
#[cfg_attr(not(test), derive(Default))]
pub struct DboxideDatabase {
  storage: Storage<Self>,

  #[cfg(test)]
  pub debug_logs: Arc<Mutex<Option<Vec<String>>>>,
}

#[cfg(test)]
impl Default for DboxideDatabase {
  fn default() -> Self {
    let debug_logs = <Arc<Mutex<Option<Vec<String>>>>>::default();
    Self {
      storage: Storage::new(Some(Box::new({
        let debug_logs = debug_logs.clone();
        move |event| {
          if let Some(debug_logs) = &mut *debug_logs.lock().unwrap()
            && let salsa::EventKind::WillExecute { .. } = event.kind
          {
            debug_logs.push(format!("Event: {event:?}"));
          }
        }
      }))),
      debug_logs,
    }
  }
}

#[db]
impl Database for DboxideDatabase {}
