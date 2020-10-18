use super::*;

#[derive(Debug)]
pub struct ExprBlock {
    collection: Vec<Expr>,

    label: Option<String>,
}
impl TryFrom<der::Block> for ExprBlock {
    type Error = PicoRuleError;

    fn try_from(ops: der::Block) -> Result<Self, Self::Error> {
        let mut this = Self {
            collection: Vec::new(),
            label: None,
        };
        for op in ops.value {
            this.collection.push(Expr::try_from(op)?)
        }
        this.label = ops.label;

        Ok(this)
    }
}

impl ExprBlock {
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
        trace!("ExprBlock");

        let mut last_collected_value: PicoValue = PicoValue::Null;

        let mut iter = self.collection.iter();
        let final_result = loop {
            if let Some(expr) = iter.next() {
                let expr_result = expr.run(ctx);
                match expr_result {
                    Ok(result) => last_collected_value = result,
                    Err(err) => match err {
                        PicoRuleError::BlockStop => break last_collected_value,
                        _ => return Err(err),
                    },
                }
            } else {
                break last_collected_value;
            }
        };

        trace!("ExprBlock end");
        Ok(final_result)
    }
}

#[derive(Debug)]
pub struct ExprStop {
    label: Option<String>,
}

impl TryFrom<der::BlockStop> for ExprStop {
    type Error = PicoRuleError;
    fn try_from(stop: der::BlockStop) -> Result<Self, Self::Error> {
        Ok(Self { label: None })
    }
}

impl ExprStop {
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
        println!("exprstop");
        trace!("ExprStop");
        Err(PicoRuleError::BlockStop)
    }
}
