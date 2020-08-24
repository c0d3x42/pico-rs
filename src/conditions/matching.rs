use serde::{Deserialize, Serialize};

use crate::commands::execution::{ConditionExecution, ConditionResult, ValueExecution};
use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
//use crate::values::{PicoValue, Var};
use crate::{PicoValue, ValueProducer};

use regex::Regex;

#[derive(Serialize, Deserialize, Debug)]
pub struct RegMatchInternal(#[serde(with = "serde_regex")] Regex, ValueProducer);

#[derive(Serialize, Deserialize, Debug)]
pub struct RegMatch {
    //    #[serde(with = "serde_regex")]
    //    regmatch: Regex,
    regmatch: RegMatchInternal, //    with: ValueProducer,
}

impl ConditionExecution for RegMatch {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        debug!("Looking up regmatch/with");

        let with_value = self.regmatch.1.run_with_context(pico_rules, runtime, ctx)?;

        match with_value {
            PicoValue::String(s) => {
                let match_result = self.regmatch.0.is_match(&s);
                trace!(
                    "Regmatch: {:?} / {:?} = {:?}",
                    self.regmatch,
                    s,
                    match_result
                );
                let re_captures = self.regmatch.0.captures(&s);
                debug!("LOC {:?}", re_captures);
                Ok(match_result)
            }
            _ => Err(PicoError::IncompatibleComparison(
                with_value,
                PicoValue::Null,
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartsWith {
    match_start: (ValueProducer, ValueProducer), // needle, haystack
}
impl ConditionExecution for StartsWith {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        let needle_value = self
            .match_start
            .0
            .run_with_context(pico_rules, runtime, ctx)?;
        let haystack_value = self
            .match_start
            .1
            .run_with_context(pico_rules, runtime, ctx)?;

        match (&needle_value, &haystack_value) {
            (PicoValue::String(needle), PicoValue::String(haystack)) => {
                let needle_str = needle.as_str();
                let haystack_str = haystack.as_str();
                let b: bool = haystack_str.starts_with(needle_str);
                Ok(b)
            }

            _ => Err(PicoError::IncompatibleComparison(
                needle_value,
                haystack_value,
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    r#match: (ValueProducer, ValueProducer),
}
impl ConditionExecution for Match {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        info!("running match");
        let lhs = self.r#match.0.run_with_context(pico_rules, runtime, ctx)?;
        let rhs = self.r#match.1.run_with_context(pico_rules, runtime, ctx)?;

        match (&lhs, &rhs) {
            (PicoValue::String(ls), PicoValue::String(rs)) => {
                let re = Regex::new(&rs).unwrap();
                let b = re.is_match(&ls);
                Ok(b)
            }
            _ => Err(PicoError::IncompatibleComparison(lhs, rhs)),
        }
    }
}
