//! Pretype analysis: resolves names and collects scope members for types and schemas only
//!
//! Entry points:
//! - `pretype_get_file_symbols`     - symbols declared/imported in a file
//! - `pretype_get_scope_members`    - symbols visible at a given scope
//! - `pretype_get_symbol_namespace` - namespace members of a type or schema
//! - `pretype_resolve_referee`      - resolve an identifier node to its symbol
//! - `get_declaration_typ`          - declared type of an element node
//! - `get_node_static_scope`        - enclosing static scope for an AST node

mod resolve;
mod scope;
mod symbol_namespace;

pub use resolve::*;
pub use scope::*;
pub use symbol_namespace::*;
