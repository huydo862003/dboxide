mod attribute;
mod constraint;
mod element_typ;
mod field_line;
mod file;
mod setting;
mod sub_element;

use std::collections::HashMap;

use itertools::Either;

use crate::db::DboxideDatabase;
use crate::db::posttype_analyze::validate::validate_file_elements::validate_file_elements;
use crate::db::test_utils::*;
use crate::db::types::interned::symbol::VirtualModuleKind;
use crate::db::types::{Diagnostics, File};
use crate::diagnostics::DiagnosticKind;

fn metatype() -> HashMap<String, Either<VirtualModuleKind, File>> {
  HashMap::from([(
    "metatype".to_string(),
    Either::Left(VirtualModuleKind::Metatype),
  )])
}
