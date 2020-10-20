use super::*;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ExprConcat {
    values: Vec<Expr>,
}

impl TryFrom<der::ConcatOp> for ExprConcat {
    type Error = PicoRuleError;

    fn try_from(s: der::ConcatOp) -> Result<Self, PicoRuleError> {
        let mut this = Self { values: Vec::new() };
        for producer in s.value {
            this.values.push(Expr::try_from(producer)?)
        }

        Ok(this)
    }
}

impl ExprConcat {
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
        trace!("ExprConcat {:?}", self.values);

        let mut collected_string: String = String::from("");

        for expr in &self.values {
            let result = expr.run(ctx)?;
            trace!("ExprConcat result {:?}", result);

            let s = json_compare::coerce_to_str(&result);

            collected_string.push_str(&s);
        }
        Ok(json!(collected_string))
    }
}
