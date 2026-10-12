use salsa::{Database, tracked};

use crate::ast::RedNode;
use crate::db::prepost::RuntimeScope;
use crate::db::types::interned::func_signature::FuncSignature;
use crate::db::types::interned::lazy_typ::LazyTyp;

use super::Typ;
use super::base::{RuntimeObj, StaticTyp};

#[tracked]
pub struct FuncTyp<'db> {
  #[returns(copy)]
  pub signature: FuncSignature<'db>,
}

impl<'db> FuncTyp<'db> {
  pub fn params(&self, db: &'db dyn Database) -> &'db Vec<LazyTyp<'db>> {
    self.signature(db).params(db)
  }

  pub fn ret(&self, db: &'db dyn Database) -> LazyTyp<'db> {
    self.signature(db).ret(db)
  }
}

impl<'db> StaticTyp<'db> for FuncTyp<'db> {
  fn display_name(&self, _db: &'db dyn Database) -> String {
    "fn".to_string()
  }

  fn runtime_typ(&self, _db: &'db dyn Database) -> Option<Typ<'db>> {
    Some((*self).into())
  }

  fn call_typ(&self, _db: &'db dyn Database) -> Option<FuncTyp<'db>> {
    Some(*self)
  }
}

impl<'db> RuntimeObj<'db> for FuncTyp<'db> {
  fn get_typ(&self, _db: &'db dyn Database) -> Typ<'db> {
    Typ::Func(*self)
  }
}

/// source is the containing AST node:
/// - for closures: the ClosureExpr node (contains params and body)
/// - for named functions: the FuncDeclaration node (contains params, return type, body)
#[tracked]
pub struct FuncObj<'db> {
  pub name: String,
  #[returns(copy)]
  pub func_typ: FuncTyp<'db>,
  pub source: Option<RedNode>,
  #[returns(copy)]
  pub captured_scope: Option<RuntimeScope<'db>>,
}

impl<'db> FuncObj<'db> {
  pub fn signature(&self, db: &'db dyn Database) -> FuncSignature<'db> {
    self.func_typ(db).signature(db)
  }
}

impl<'db> RuntimeObj<'db> for FuncObj<'db> {
  fn get_typ(&self, db: &'db dyn Database) -> Typ<'db> {
    Typ::Func(self.func_typ(db))
  }

  fn to_display_string(&self, db: &'db dyn Database) -> String {
    self.name(db).clone()
  }
}
