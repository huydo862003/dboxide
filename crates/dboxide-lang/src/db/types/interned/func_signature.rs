use salsa::interned;

use crate::db::types::tracked::obj_system::Typ;

#[interned]
pub struct FuncSignature<'db> {
  #[returns(ref)]
  pub params: Vec<Typ<'db>>,
  #[returns(copy)]
  pub ret: Typ<'db>,
}
