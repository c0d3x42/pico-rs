use super::*;
use json_compare::*;

#[derive(Debug)]
pub struct ExprGt {
  lhs: Box<Expr>,
  rhs: Vec<Expr>,
}

impl Default for ExprGt {
  fn default() -> Self {
    Self {
      lhs: Box::new(Expr::Nop),
      rhs: Vec::new(),
    }
  }
}

impl TryFrom<der::GreaterThanOperation> for ExprGt {
  type Error = PicoRuleError;

  fn try_from(gt_operation: der::GreaterThanOperation) -> Result<ExprGt, Self::Error> {
    trace!("ExprGt::TryFrom {:?}", gt_operation.value);
    let mut this = Self::default();

    // must have at least two componnets
    if gt_operation.value.len() < 2 {
      return Err(PicoRuleError::InvalidPicoRule);
    }

    let mut iter = gt_operation.value.into_iter();

    if let Some(expr_first) = iter.next() {
      this.lhs = Box::new(Expr::try_from(expr_first)?);

      for expr in iter {
        this.rhs.push(Expr::try_from(expr)?);
      }
    }
    trace!("ExpLt::TryFrom {:?}", this);
    Ok(this)
  }
}

impl ExprGt {
  pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
    let mut left = self.lhs.run(ctx).unwrap_or(PicoValue::Null);

    trace!("ExprGt {:?}, {:?}", self.lhs, self.rhs);

    for val in &self.rhs {
      let right = val.run(ctx)?;
      trace!("ExprGt {} < {}", left, right);
      if json_compare::greater_than(&left, &right) {
        left = right;
      } else {
        return Ok(PicoValue::Bool(false));
      }
    }

    Ok(PicoValue::Bool(true))
  }
}
