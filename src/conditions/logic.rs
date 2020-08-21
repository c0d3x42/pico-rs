use serde::{Deserialize, Serialize};

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::conditions::Condition;
use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::loader::PicoRules;
use crate::loader::PicoRuntime as PicoState;
//use crate::values::{PicoValue, Var};
use crate::PicoValue;

/*
 * condition collections
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct And {
    and: Vec<Condition>,
}

impl Execution for And {
    fn name(&self) -> String {
        "and".to_string()
    }
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        state: &mut PicoState,
        ctx: &mut PicoContext,
    ) -> FnResult {
        for condition in &self.and {
            let condition_result = condition.run_with_context(pico_rules, state, ctx)?;

            match condition_result {
                ExecutionResult::Stop(stopping_reason) => {
                    return Ok(ExecutionResult::Stop(stopping_reason))
                }
                ExecutionResult::Continue(continuation) => match continuation {
                    PicoValue::Boolean(b) => {
                        if !b {
                            // AND exits as soon as one condition returns boolean false
                            return Ok(ExecutionResult::Continue(PicoValue::Boolean(false)));
                        }
                    }
                    _ => return Err(PicoError::Crash("non boolean".to_string())),
                },
                c => return Ok(c),
            }
        }
        // all conditions returned boolean true
        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Or {
    or: Vec<Condition>,
}
impl Execution for Or {
    fn name(&self) -> String {
        "or".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        state: &mut PicoState,
        ctx: &mut PicoContext,
    ) -> FnResult {
        let condition_count = self.or.len();
        debug!("OR ...{:?}", condition_count);

        for condition in &self.or {
            let condition_result = condition.run_with_context(pico_rules, state, ctx)?;

            match condition_result {
                ExecutionResult::Stop(stopping) => return Ok(ExecutionResult::Stop(stopping)),
                ExecutionResult::Continue(continuation) => match continuation {
                    PicoValue::Boolean(b) => {
                        if b {
                            // OR completes succesfully on the first boolean true
                            return Ok(ExecutionResult::Continue(PicoValue::Boolean(true)));
                        }
                    }
                    _ => return Err(PicoError::Crash("Non boolean".to_string())),
                },
                c => return Ok(c),
            }
        }
        Ok(ExecutionResult::Continue(PicoValue::Boolean(false)))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Not {
    not: Box<Condition>,
}
impl Execution for Not {
    fn name(&self) -> String {
        "not".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        state: &mut PicoState,
        ctx: &mut PicoContext,
    ) -> FnResult {
        let condition_result = self.not.run_with_context(pico_rules, state, ctx)?;

        match condition_result {
            ExecutionResult::Continue(val) => match val {
                PicoValue::Boolean(b) => Ok(ExecutionResult::Continue(PicoValue::Boolean(!b))),
                _ => Err(PicoError::IncompatibleComparison),
            },
            c => Ok(c), //ExecutionResult::Stop(s) => return Ok(ExecutionResult::Stop(s)),
        }
    }
}
