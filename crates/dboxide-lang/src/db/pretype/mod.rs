pub(crate) mod pretype_evaluate;
pub(crate) mod pretype_get_file_usable_symbols;
pub(crate) mod pretype_get_scope_members;
pub(crate) mod pretype_get_symbol_namespace;
pub(crate) mod pretype_resolve_name;

#[cfg(test)]
mod tests;

pub use pretype_evaluate::evaluate_node;
