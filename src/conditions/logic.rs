use serde::{Deserialize, Serialize};

use crate::commands::execution::{ConditionExecution, ConditionResult};
use crate::conditions::Condition;
use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
//use crate::values::{PicoValue, Var};
use crate::PicoValue;

/*
 * condition collections
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct And {
    and: Vec<Condition>,
}

impl ConditionExecution for And {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        for condition in &self.and {
            let condition_result = condition.run_with_context(pico_rules, runtime, ctx)?;
            if condition_result == false {
                return Ok(false);
            }
        }
        // all conditions returned boolean true
        Ok(true)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Or {
    or: Vec<Condition>,
}
impl ConditionExecution for Or {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        let condition_count = self.or.len();
        debug!("OR ...{:?}", condition_count);

        for condition in &self.or {
            let condition_result = condition.run_with_context(pico_rules, runtime, ctx)?;

            if condition_result == true {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Not {
    not: Box<Condition>,
}
impl ConditionExecution for Not {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        let condition_result = self.not.run_with_context(pico_rules, runtime, ctx)?;

        Ok(!condition_result)
    }
}
