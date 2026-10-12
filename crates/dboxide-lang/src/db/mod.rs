//! Salsa database and incremental queries
//!
//! Pipeline stages:
//! 1. syntax: parse files, extract use specifiers
//! 2. pretype: type evaluation, scope (types only), symbol namespaces
//! 3. posttype: full evaluation, scope (all symbols), checking, validation
//!
//! Import resolution is host-driven:
//! 1. Host creates `File` inputs with content
//! 2. `parse_file` produces the AST (cached by salsa)
//! 3. `get_use_specifiers` extracts unresolved specifiers from the AST
//! 4. Host resolves each specifier and sets `File.uses`
//! 5. `get_scope_members` reads `File.uses` to merge imported symbols
//!
//! The host owns specifier resolution (relative paths, import maps, etc).
//! The language never touches the filesystem.

#[cfg(test)]
use std::sync::{Arc, Mutex};

#[cfg(test)]
pub(crate) mod test_utils;

use salsa::{Database, Storage, db};

mod posttype;
mod prepost;
mod pretype;
mod types;

pub use pretype::*;
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
