use super::*;

#[derive(Debug)]
pub struct ExprLt {
  lhs: Box<Expr>,
  rhs: Vec<Expr>,
}

impl Default for ExprLt {
  fn default() -> Self {
    Self {
      lhs: Box::new(Expr::Nop),
      rhs: Vec::new(),
    }
  }
}

impl TryFrom<der::LessThanOperation> for ExprLt {
  type Error = PicoRuleError;

  fn try_from(lt_operation: der::LessThanOperation) -> Result<ExprLt, Self::Error> {
    let mut this = Self::default();

    // must have at least two componnets
    if lt_operation.value.len() < 2 {
      return Err(PicoRuleError::InvalidPicoRule);
    }

    let mut iter = lt_operation.value.into_iter();

    if let Some(expr_first) = iter.next() {
      this.lhs = Box::new(Expr::try_from(expr_first)?);

      for expr in iter {
        this.rhs.push(Expr::try_from(expr)?);
      }
    }
    Ok(this)
  }
}

impl ExprLt {
  pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
    let mut left = self.lhs.run(ctx).unwrap_or(PicoValue::Null);

    trace!("ExprLt {:?}, {:?}", self.lhs, self.rhs);

    for val in &self.rhs {
      let right = val.run(ctx)?;
      if logic::less_than(&left, &right) {
        left = right;
      } else {
        return Ok(PicoValue::Bool(false));
      }
    }

    Ok(PicoValue::Bool(true))
  }
}
