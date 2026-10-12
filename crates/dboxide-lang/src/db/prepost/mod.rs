pub(crate) mod evaluate_node;
pub(crate) mod get_file_symbols;
pub(crate) mod get_scope_members;
pub(crate) mod get_symbol_namespace;
pub(crate) mod resolve_name;
mod utils;

pub(crate) use utils::runtime_scope::RuntimeScope;

pub use resolve_name::{resolve_name_posttype, resolve_name_pretype};
pub use utils::*;
