use crate::db::types::input::File;
use crate::db::types::interned::symbol::VirtualModuleKind;

/// What a `from` clause resolves to
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Module {
  Virtual(VirtualModuleKind),
  File(File),
}
