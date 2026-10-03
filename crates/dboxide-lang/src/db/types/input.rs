use std::{collections::HashMap, path::PathBuf, time::SystemTime};

use salsa::input;

use crate::FileHandle;

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
}

#[input]
pub struct Project {
  #[returns(ref)]
  pub files: HashMap<PathBuf, File>,
}
