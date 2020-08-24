use serde::{Deserialize, Serialize};

use crate::commands::execution::{ConditionExecution, ConditionResult, ValueExecution};
use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
//use crate::values::{PicoValue, Var};
use crate::{PicoValue, ValueProducer};

#[derive(Serialize, Deserialize, Debug)]
pub struct Eq {
    eq: (ValueProducer, ValueProducer),
}
impl ConditionExecution for Eq {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        trace!("Eq resolving...");
        let lhs = self.eq.0.run_with_context(pico_rules, runtime, ctx)?;
        let rhs = self.eq.1.run_with_context(pico_rules, runtime, ctx)?;
        trace!("LHS = {:?}", lhs);
        trace!("RHS = {:?}", rhs);

        Ok(lhs == rhs)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GreaterThan {
    gt: (ValueProducer, ValueProducer),
}
impl ConditionExecution for GreaterThan {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        let lhs = self.gt.0.run_with_context(pico_rules, runtime, ctx)?;
        let rhs = self.gt.1.run_with_context(pico_rules, runtime, ctx)?;

        match (&lhs, &rhs) {
            (PicoValue::Number(left), PicoValue::Number(right)) => {
                trace!("{} > {}", left, right);
                match (left.as_i64(), right.as_i64()) {
                    (Some(l), Some(r)) => Ok(l > r),
                    _ => Err(PicoError::IncompatibleComparison(lhs, rhs)),
                }
            }
            _ => {
                info!("cant compare {} > {}", lhs, rhs);
                Err(PicoError::IncompatibleComparison(lhs, rhs))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LessThan {
    lt: (ValueProducer, ValueProducer),
}
impl ConditionExecution for LessThan {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        let lhs = self.lt.0.run_with_context(pico_rules, runtime, ctx)?;
        let rhs = self.lt.1.run_with_context(pico_rules, runtime, ctx)?;

        match (&lhs, &rhs) {
            (PicoValue::Number(left), PicoValue::Number(right)) => {
                trace!("{} < {}", left, right);

                match (left.as_i64(), right.as_i64()) {
                    (Some(l), Some(r)) => Ok(l < r),
                    _ => Err(PicoError::IncompatibleComparison(lhs, rhs)),
                }
            }
            (PicoValue::String(left), PicoValue::String(right)) => Ok(left < right),
            _ => {
                info!("cant compare {} > {}", lhs, rhs);
                Err(PicoError::IncompatibleComparison(lhs, rhs))
            }
        }
    }
}
