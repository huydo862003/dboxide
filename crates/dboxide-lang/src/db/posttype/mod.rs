pub(crate) mod posttype_check_element;
pub(crate) mod posttype_check_file;
pub(crate) mod posttype_declare_element;
pub(crate) mod posttype_evaluate_node;
pub(crate) mod posttype_get_file_symbols;
pub(crate) mod posttype_get_scope_members;
pub(crate) mod posttype_resolve_name;
pub(crate) mod posttype_resolve_reference;
pub(crate) mod posttype_validate_constraints;
pub(crate) mod runtime_scope;

#[cfg(test)]
mod tests;
