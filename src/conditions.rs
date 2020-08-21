use serde::{Deserialize, Serialize};

pub mod compare;
pub mod existence;
pub mod logic;
pub mod matching;

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::conditions::compare::{Eq, GreaterThan, LessThan};
use crate::conditions::existence::{VarExistsCondition, VarMissingCondition};
use crate::conditions::logic::{And, Not, Or};
use crate::conditions::matching::{Match, RegMatch, StartsWith};

use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::loader::PicoRules;
use crate::loader::PicoRuntime as PicoState;
//use crate::values::{PicoValue, Var};
use crate::PicoValue;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Condition {
    And(And),
    Or(Or),
    Eq(Eq),
    Match(Match),
    RegMatch(RegMatch),
    StartsWith(StartsWith),
    GreaterThan(GreaterThan),
    LessThan(LessThan),
    VarExists(VarExistsCondition),
    VarMissing(VarMissingCondition),
    Not(Not),
}

impl Execution for Condition {
    fn name(&self) -> String {
        "condition".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        state: &mut PicoState,
        ctx: &mut PicoContext,
    ) -> FnResult {
        debug!("Checking condition {:?}", self);
        let condition_result = match self {
            Condition::And(and) => and.run_with_context(pico_rules, state, ctx),
            Condition::Or(or) => or.run_with_context(pico_rules, state, ctx),
            Condition::Not(not) => not.run_with_context(pico_rules, state, ctx),
            Condition::Match(m) => m.run_with_context(pico_rules, state, ctx),
            Condition::RegMatch(rm) => rm.run_with_context(pico_rules, state, ctx),
            Condition::StartsWith(sw) => sw.run_with_context(pico_rules, state, ctx),

            Condition::Eq(eq) => eq.run_with_context(pico_rules, state, ctx),
            Condition::GreaterThan(gt) => gt.run_with_context(pico_rules, state, ctx),
            Condition::LessThan(lt) => lt.run_with_context(pico_rules, state, ctx),

            Condition::VarExists(ve) => ve.run_with_context(pico_rules, state, ctx),
            Condition::VarMissing(vm) => vm.run_with_context(pico_rules, state, ctx),
        };

        match condition_result {
            Ok(result) => Ok(result),
            Err(error_result) => match error_result {
                PicoError::NoSuchValue(_) | PicoError::IncompatibleComparison => {
                    info!("condition result was bad - mapping to false");
                    Ok(ExecutionResult::Continue(PicoValue::Boolean(false)))
                }
                err => Err(err),
            },
        }
    }
}
