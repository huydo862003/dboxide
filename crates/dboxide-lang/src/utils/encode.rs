use crate::ast::{AstNode, FuncDeclaration};

/// Encodes the type signature of an operator declaration into a string suitable for use as a unique symbol key
/// Produces `{op_symbol}({param_type1},{param_type2},...)`
pub fn encode_operator_signature(func: &FuncDeclaration) -> String {
  let op_symbol = func
    .name()
    .and_then(|name| name.operator_symbol())
    .unwrap_or_default();

  let param_types: Vec<String> = func
    .params()
    .into_iter()
    .flat_map(|param| param.params().collect::<Vec<_>>())
    .map(|param| {
      param
        .typ_expr()
        .map(|expr| expr.text().trim().to_string())
        .unwrap_or_default()
    })
    .collect();

  format!("{op_symbol}({})", param_types.join(","))
}
