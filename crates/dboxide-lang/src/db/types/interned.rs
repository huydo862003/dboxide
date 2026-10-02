use std::path::PathBuf;

use salsa::interned;

#[interned]
pub struct Filepath {
  #[returns(deref)]
  value: PathBuf,
}
