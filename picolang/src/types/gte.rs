use super::*;
use json_compare::*;

#[derive(Debug)]
pub struct ExprGtE {
  lhs: Box<Expr>,
  rhs: Vec<Expr>,
}

impl Default for ExprGtE {
  fn default() -> Self {
    Self {
      lhs: Box::new(Expr::Nop),
      rhs: Vec::new(),
    }
  }
}

impl TryFrom<der::GreaterThanEqualOperation> for ExprGtE {
  type Error = PicoRuleError;

  fn try_from(gte_operation: der::GreaterThanEqualOperation) -> Result<ExprGtE, Self::Error> {
    trace!("ExprGtE::TryFrom {:?}", gte_operation.value);
    let mut this = Self::default();

    // must have at least two componnets
    if gte_operation.value.len() < 2 {
      return Err(PicoRuleError::InvalidPicoRule);
    }

    let mut iter = gte_operation.value.into_iter();

    if let Some(expr_first) = iter.next() {
      this.lhs = Box::new(Expr::try_from(expr_first)?);

      for expr in iter {
        this.rhs.push(Expr::try_from(expr)?);
      }
    }
    trace!("ExpGtE::TryFrom {:?}", this);
    Ok(this)
  }
}

impl ExprGtE {
  pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
    let mut left = self.lhs.run(ctx).unwrap_or(PicoValue::Null);

    trace!("ExprGtE::exec {:?}, {:?}", self.lhs, self.rhs);

    for val in &self.rhs {
      let right = val.run(ctx)?;
      trace!("ExprGtE {} <= {}", left, right);
      if json_compare::greater_equal_than(&left, &right) {
        left = right;
      } else {
        return Ok(PicoValue::Bool(false));
      }
    }

    Ok(PicoValue::Bool(true))
  }
}
