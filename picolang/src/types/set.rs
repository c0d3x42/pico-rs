use super::*;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ExprSet {
    varbind: String,
    value: PicoValue,
}

impl TryFrom<der::SetStmt> for ExprSet {
    type Error = PicoRuleError;

    fn try_from(s: der::SetStmt) -> Result<Self, PicoRuleError> {
        let stmt = Self {
            varbind: s.value.0,
            value: s.value.1,
        };

        Ok(stmt)
    }
}

impl ExprSet {
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
        trace!("ExprSet {}", self.varbind);

        match ctx.insert(&self.varbind, &self.value) {
            None => trace!("inserted {}, {}", self.varbind, self.value),
            Some(v) => trace!("updated {}, {}", self.varbind, self.value),
        }

        Ok(json!(self.varbind))
    }
}
