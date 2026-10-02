use salsa::accumulator;

use crate::diagnostics::Diagnostic;

#[accumulator]
pub struct Diagnostics(Diagnostic);
