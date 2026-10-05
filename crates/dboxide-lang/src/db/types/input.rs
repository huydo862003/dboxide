use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use itertools::Either;
use salsa::input;

use crate::{FileHandle, VirtualModuleKind};
#[input]
pub struct File {
  #[returns(deref)]
  pub filepath: PathBuf,

  #[returns(ref)]
  pub handle: FileHandle,

  #[returns(copy)]
  pub ctime: SystemTime,

  #[returns(copy)]
  pub mtime: SystemTime,

  /// Host-resolved imports: specifier string -> resolved Module
  #[returns(ref)]
  pub uses: HashMap<String, Either<VirtualModuleKind, File>>,
}
