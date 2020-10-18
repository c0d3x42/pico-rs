
use super::*;

#[derive(Debug)]
pub struct ExprAnd {
    and: Vec<Expr>
}
impl TryFrom<der::AndOperation> for ExprAnd {
    type Error = PicoRuleError;

    fn try_from(andop: der::AndOperation) -> Result<Self, Self::Error> {

        let mut this = Self{ and: Vec::new()};
        for op in andop.value   {
            this.and.push(Expr::try_from(op)?)
        }

        Ok(this)
    }
}

impl ExprAnd{

    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError>{
        trace!("ExprAnd");

        let mut last :Option<PicoValue> = None;

        for a in &self.and {
            let result = a.run(ctx)?;

            if pico_value_as_truthy(&result){
                trace!("ExprAnd!");
                last = Some(result);
            } else {
                return Ok(result);
            }
        }

        trace!("ExprAnd end");
        Ok(last.unwrap_or(PicoValue::Null))
    }
}