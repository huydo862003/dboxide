//! Posttype analysis: resolves names and collects scope members for all symbols
//!
//! Entry points:
//! - `posttype_get_file_symbols`         - symbols declared/imported in a file (extends pretype with elements, fns, operators)
//! - `posttype_get_scope_members`        - symbols visible at a given scope
//! - `posttype_get_symbol_namespace`     - namespace members of a symbol (extends pretype with element children)
//! - `posttype_resolve_referee`          - resolve an identifier node to its symbol
//! - `posttype_resolve_referee_candidates` - collect all candidate symbols for a chain node
//! - `get_node_expected_typ`             - expected type at a node position (for type inference)
//! - `get_node_actual_typ`               - actual type of a node position

mod resolve;
mod scope;
mod symbol_namespace;
mod typecheck;
mod validate;

pub use resolve::*;
pub use scope::*;
pub use symbol_namespace::*;
pub use typecheck::get_node_expected_typ;
pub use validate::validate_file_elements::validate_file_elements;
