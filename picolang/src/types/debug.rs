use super::*;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ExprDebug  {
  msg: String,
}

impl TryFrom<der::DebugStmt> for ExprDebug {
  type Error = PicoRuleError;

  fn try_from( d: der::DebugStmt) -> Result<Self, PicoRuleError>{
    let debug = Self {
      msg: d.value
    };

    Ok(debug)

  }
}

impl ExprDebug {

  pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
    trace!("ExprDebug");

    debug!("ExprDebug {}", self.msg);

    let s = format!("DEBUG {}", self.msg);
    ctx.add_msg(&s);

    Ok(PicoValue::Null)
  }
}
