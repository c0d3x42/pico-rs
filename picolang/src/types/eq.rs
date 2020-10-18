use super::*;
/**
 * ExprEq
 *
 */
#[derive(Debug)]
pub struct ExprEq {
    lhs: Box<Expr>,
    rhs: Box<Expr>,
}
impl TryFrom<der::EqOperation> for ExprEq {
    type Error = PicoRuleError;

    fn try_from(eq_operation: der::EqOperation) -> Result<ExprEq, Self::Error> {
        Ok(Self {
            lhs: Box::new(Expr::try_from(eq_operation.value.0)?),
            rhs: Box::new(Expr::try_from(eq_operation.value.1)?),
        })
    }
}

impl ExprEq{

    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError>{
        trace!("ExprEq...");
        let left_hand_value = self.lhs.run(ctx)?;
        let right_hand_value = self.rhs.run(ctx)?;
        trace!("ExprEq {} == {}", left_hand_value, right_hand_value);

        Ok( PicoValue::Bool(logic::equality(&left_hand_value, &right_hand_value) ) )
    }
}