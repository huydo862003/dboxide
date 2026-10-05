mod metatype;

use std::collections::BTreeMap;

use salsa::Database;

use super::types::interned::symbol::VirtualModuleKind;
use super::types::{Module, Symbol};

pub fn resolve_module(source: &str) -> Option<Module> {
  match source {
    "metatype" => Some(Module::Virtual(VirtualModuleKind::Metatype)),
    // TODO: resolve file paths to Module::File
    _ => None,
  }
}

pub fn get_module_exports(db: &dyn Database, module: Module) -> BTreeMap<String, Symbol<'_>> {
  match module {
    Module::Virtual(VirtualModuleKind::Metatype) => metatype::get_metatype_members(db),
    // TODO: collect file exports
    Module::File(_file) => BTreeMap::new(),
  }
}

pub(crate) use metatype::*;
