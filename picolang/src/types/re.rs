use super::*;
use std::convert::TryFrom;
use regex::Regex;

#[derive(Debug)]
pub struct ExprRegex {
    re: regex::Regex,
    expr: Box<Expr>,
}

impl TryFrom<der::RegexOperation> for ExprRegex {
    type Error = PicoRuleError;

    fn try_from(r: der::RegexOperation) -> Result<Self, PicoRuleError> {
        let mut iter = r.value.into_iter();

        if let Some( p ) = iter.next() {
            trace!("RegexOperation: iter {:?}", p);
            let re = match p {
                der::Producer::String(s) => Regex::new(&s).map_err(|_| PicoRuleError::InvalidPicoRule)?,
                _ => return Err(PicoRuleError::InvalidPicoRule)
            };

            let expr = match iter.next() {
                Some( p) => Expr::try_from(p)?,
                None => return Err(PicoRuleError::InvalidPicoRule)
            };

            return Ok(Self{re, expr: Box::new(expr)})
        }
        Err(PicoRuleError::InvalidPicoRule)
    }
}

impl ExprRegex {
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
        trace!("ExprRegex {}, {:?}", self.re, self.expr);

        let expr_result = self.expr.run(ctx)?;
        trace!("ExprRegexResult {}", expr_result);

        let result = if let Some(haystack) = expr_result.as_str(){
            self.re.is_match(haystack)
        } else {
            false
        };

        Ok(json!(result))
    }
}
