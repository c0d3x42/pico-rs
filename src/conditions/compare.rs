use serde::{Deserialize, Serialize};

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::context::PicoContext;
use crate::state::PicoState;
//use crate::values::{PicoValue, Var};
use crate::{PicoValue, ValueProducer};

#[derive(Serialize, Deserialize, Debug)]
pub struct Eq {
    eq: (ValueProducer, ValueProducer),
}
impl Execution for Eq {
    fn name(&self) -> String {
        "equality".to_string()
    }
    fn run_with_context(&self, state: &mut PicoState, ctx: &mut PicoContext) -> FnResult {
        trace!("Eq resolving...");
        let lhs = self.eq.0.run_with_context(state, ctx)?;
        let rhs = self.eq.1.run_with_context(state, ctx)?;
        trace!("LHS = {:?}", lhs);
        trace!("RHS = {:?}", rhs);

        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                Ok(ExecutionResult::Continue(PicoValue::Boolean(left == right)))
            }

            _ => Ok(ExecutionResult::Continue(PicoValue::Boolean(false))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GreaterThan {
    gt: (ValueProducer, ValueProducer),
}
impl Execution for GreaterThan {
    fn name(&self) -> String {
        "less than".to_string()
    }
    fn run_with_context(&self, state: &mut PicoState, ctx: &mut PicoContext) -> FnResult {
        let lhs = self.gt.0.run_with_context(state, ctx)?;
        let rhs = self.gt.1.run_with_context(state, ctx)?;
        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                Ok(ExecutionResult::Continue(PicoValue::Boolean(left > right)))
            }
            _ => Ok(ExecutionResult::Continue(PicoValue::Boolean(false))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LessThan {
    lt: (ValueProducer, ValueProducer),
}
impl Execution for LessThan {
    fn name(&self) -> String {
        "less than".to_string()
    }
    fn run_with_context(&self, state: &mut PicoState, ctx: &mut PicoContext) -> FnResult {
        let lhs = self.lt.0.run_with_context(state, ctx)?;
        let rhs = self.lt.1.run_with_context(state, ctx)?;
        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                Ok(ExecutionResult::Continue(PicoValue::Boolean(left < right)))
            }
            _ => Ok(ExecutionResult::Continue(PicoValue::Boolean(false))),
        }
    }
}
