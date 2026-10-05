use salsa::interned;

use crate::db::types::interned::lazy_typ::LazyTyp;

#[interned]
pub struct FuncSignature<'db> {
  #[returns(ref)]
  pub params: Vec<LazyTyp<'db>>,
  #[returns(copy)]
  pub ret: LazyTyp<'db>,
}
