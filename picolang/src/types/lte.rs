use super::*;
use json_compare::*;

#[derive(Debug)]
pub struct ExprLtE {
  lhs: Box<Expr>,
  rhs: Vec<Expr>,
}

impl Default for ExprLtE {
  fn default() -> Self {
    Self {
      lhs: Box::new(Expr::Nop),
      rhs: Vec::new(),
    }
  }
}

impl TryFrom<der::LessThanEqualOperation> for ExprLtE {
  type Error = PicoRuleError;

  fn try_from(lte_operation: der::LessThanEqualOperation) -> Result<ExprLtE, Self::Error> {
    trace!("ExprLtE::TryFrom {:?}", lte_operation.value);
    let mut this = Self::default();

    // must have at least two componnets
    if lte_operation.value.len() < 2 {
      return Err(PicoRuleError::InvalidPicoRule);
    }

    let mut iter = lte_operation.value.into_iter();

    if let Some(expr_first) = iter.next() {
      this.lhs = Box::new(Expr::try_from(expr_first)?);

      for expr in iter {
        this.rhs.push(Expr::try_from(expr)?);
      }
    }
    trace!("ExpLtE::TryFrom {:?}", this);
    Ok(this)
  }
}

impl ExprLtE {
  pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
    let mut left = self.lhs.run(ctx).unwrap_or(PicoValue::Null);

    trace!("ExprLtE::exec {:?}, {:?}", self.lhs, self.rhs);

    for val in &self.rhs {
      let right = val.run(ctx)?;
      trace!("ExprLt {} < {}", left, right);
      if json_compare::less_equal_than(&left, &right) {
        left = right;
      } else {
        return Ok(PicoValue::Bool(false));
      }
    }

    Ok(PicoValue::Bool(true))
  }
}
