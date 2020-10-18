use super::*;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ExprOr {
    or: Vec<Expr>
}
impl TryFrom<der::OrOperation> for ExprOr {
    type Error = PicoRuleError;

    fn try_from(orop: der::OrOperation) -> Result<Self, Self::Error> {

        let mut this = Self{ or: Vec::new()};
        for op in orop.value   {
            this.or.push(Expr::try_from(op)?)
        }

        Ok(this)
    }
}

impl ExprOr{

    /**
     * or returns the first truthy argument, or the last argument
     */
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError>{
        trace!("ExprOr");

        let mut last : Option<PicoValue> = None;

        for a in &self.or {
            let result = a.run(ctx)?;

            if pico_value_as_truthy(&result){
              return Ok(result);
            } 
            trace!("ExprOr!");
            last = Some(result);
        }

        trace!("ExprOr end");
        Ok(last.unwrap_or(PicoValue::Null))
    }
}