use super::*;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ExprLet  {
  varbind: String,
  value: Box<Expr>
}

impl TryFrom<der::LetStmt> for ExprLet {
  type Error = PicoRuleError;

  fn try_from( l: der::LetStmt) -> Result<Self, PicoRuleError>{
    let stmt = Self {
      varbind: l.value.0,
      value: Box::new(Expr::try_from(*l.value.1)?)
    };

    Ok(stmt)

  }
}


impl ExprLet {

  pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
    trace!("ExprLet {}", self.varbind);
    Ok(PicoValue::Bool(true))
  }
}
