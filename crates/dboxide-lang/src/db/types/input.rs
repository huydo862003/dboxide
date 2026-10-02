use std::{collections::HashMap, path::PathBuf, time::SystemTime};

use salsa::input;

use crate::FileHandle;

#[input]
pub struct File {
  #[returns(deref)]
  filepath: PathBuf,

  #[returns(ref)]
  handle: FileHandle,

  #[returns(copy)]
  ctime: SystemTime,

  #[returns(copy)]
  mtime: SystemTime,
}

#[input]
pub struct Project {
  #[returns(ref)]
  files: HashMap<PathBuf, File>,
}
