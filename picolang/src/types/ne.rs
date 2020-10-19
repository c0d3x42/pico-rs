use super::*;
use json_compare::is_abstract_equal;
/**
 * ExprNe
 *
 */
#[derive(Debug)]
pub struct ExprNe {
    lhs: Box<Expr>,
    rhs: Box<Expr>,
}
impl TryFrom<der::NeOperation> for ExprNe {
    type Error = PicoRuleError;

    fn try_from(ne_operation: der::NeOperation) -> Result<ExprNe, Self::Error> {
        Ok(Self {
            lhs: Box::new(Expr::try_from(ne_operation.value.0)?),
            rhs: Box::new(Expr::try_from(ne_operation.value.1)?),
        })
    }
}

impl ExprNe {
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
        trace!("ExprNe...");
        let left_hand_value = self.lhs.run(ctx)?;
        let right_hand_value = self.rhs.run(ctx)?;
        trace!("ExprNe {} == {}", left_hand_value, right_hand_value);

        Ok(PicoValue::Bool(!is_abstract_equal(
            &left_hand_value,
            &right_hand_value,
        )))
    }
}
