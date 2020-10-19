use super::*;
use json_compare::is_truthy;

#[derive(Debug)]
pub struct PicoIf {
  r#if_then: Vec<(Expr, Expr)>,
  r#else: Option<Expr>,
}

impl Default for PicoIf {
  fn default() -> Self {
    Self {
      if_then: Vec::new(),
      r#else: None,
    }
  }
}

impl TryFrom<der::IfStmt> for PicoIf {
  type Error = PicoRuleError;

  fn try_from(if_operation: der::IfStmt) -> Result<PicoIf, Self::Error> {
    let mut this = Self::default();

    if if_operation.value.len() < 2 {
      return Err(PicoRuleError::InvalidPicoRule);
    }

    let mut iter = if_operation.value.into_iter().peekable();

    while let Some(expr1) = iter.next() {
      if iter.peek().is_some() {
        if let Some(expr2) = iter.next() {
          this
            .if_then
            .push((Expr::try_from(expr1)?, Expr::try_from(expr2)?));
        }
      } else {
        this.r#else = Some(Expr::try_from(expr1)?);
      }
    }

    Ok(this)
    //Err(PicoRuleError::InvalidPicoRule)
  }
}

impl PicoIf {
  pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
    trace!("PicoIf");

    for (if_stmt, then_stmt) in &self.if_then {
      let res = if_stmt.run(ctx)?;
      if is_truthy(&res) {
        trace!("PicoIf..Then");
        return then_stmt.run(ctx);
      }
    }

    if let Some(else_stmt) = &self.r#else {
      trace!("PicoIf..Else {:?}", else_stmt);
      return else_stmt.run(ctx);
    }

    trace!("PicoIf not matched");

    Ok(PicoValue::Null)
  }
}
