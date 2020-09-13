use serde::{Deserialize, Serialize};

pub mod compare;
pub mod existence;
pub mod logic;
pub mod matching;

use crate::commands::execution::{ConditionExecution, ConditionResult};
use crate::conditions::compare::{Eq, GreaterThan, LessThan};
use crate::conditions::existence::{VarExistsCondition, VarMissingCondition};
use crate::conditions::logic::{And, Not, Or};
use crate::conditions::matching::{Match, RegMatch, StartsWith};

use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
//use crate::values::{PicoValue, Var};

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

impl ConditionExecution for Condition {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        debug!("Checking condition {:?}", self);
        let condition_result = match self {
            Condition::And(and) => and.run_with_context(pico_rules, runtime, ctx),
            Condition::Or(or) => or.run_with_context(pico_rules, runtime, ctx),
            Condition::Not(not) => not.run_with_context(pico_rules, runtime, ctx),
            Condition::Match(m) => m.run_with_context(pico_rules, runtime, ctx),
            Condition::RegMatch(rm) => rm.run_with_context(pico_rules, runtime, ctx),
            Condition::StartsWith(sw) => sw.run_with_context(pico_rules, runtime, ctx),

            Condition::Eq(eq) => eq.run_with_context(pico_rules, runtime, ctx),
            Condition::GreaterThan(gt) => gt.run_with_context(pico_rules, runtime, ctx),
            Condition::LessThan(lt) => lt.run_with_context(pico_rules, runtime, ctx),

            Condition::VarExists(ve) => ve.run_with_context(pico_rules, runtime, ctx),
            Condition::VarMissing(vm) => vm.run_with_context(pico_rules, runtime, ctx),
        };

        match condition_result {
            Ok(result) => Ok(result),
            Err(error_result) => match error_result {
                PicoError::IncompatibleComparison(lhs, rhs) => {
                    warn!("cant compare {} with {}", lhs, rhs);
                    Ok(false)
                }
                PicoError::NoSuchValue(_) => {
                    warn!("no such value - mapping to false: {}", error_result);
                    Ok(false)
                }
                err => Err(err),
            },
        }
    }
}
