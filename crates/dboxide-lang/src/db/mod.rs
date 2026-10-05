#[cfg(test)]
use std::sync::{Arc, Mutex};

use salsa::{Database, Storage, db};

pub mod name_resolve;
pub mod parse_file;
mod types;

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
          // Log interesting events, if logging is enabled
          if let Some(debug_logs) = &mut *debug_logs.lock().unwrap() {
            // only log interesting events
            if let salsa::EventKind::WillExecute { .. } = event.kind {
              debug_logs.push(format!("Event: {event:?}"));
            }
          }
        }
      }))),
      debug_logs,
    }
  }
}

#[db]
impl Database for DboxideDatabase {}
