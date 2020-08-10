use serde::{Deserialize, Serialize};

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::context::PicoContext;
use crate::errors::PicoError;
use crate::state::PicoState;
//use crate::values::{PicoValue, Var};
use crate::{PicoValue, ValueProducer};

use regex::Regex;
use serde_regex;

#[derive(Serialize, Deserialize, Debug)]
pub struct RegMatchInternal(#[serde(with = "serde_regex")] Regex, ValueProducer);

#[derive(Serialize, Deserialize, Debug)]
pub struct RegMatch {
    //    #[serde(with = "serde_regex")]
    //    regmatch: Regex,
    regmatch: RegMatchInternal, //    with: ValueProducer,
}

impl Execution for RegMatch {
    fn name(&self) -> String {
        return "regmatch".to_string();
    }

    fn run_with_context(&self, state: &mut PicoState, ctx: &mut PicoContext) -> FnResult {
        debug!("Looking up regmatch/with");

        let with_value = self.regmatch.1.run_with_context(state, ctx)?;

        match with_value {
            ExecutionResult::Stop(stopping_reason) => {
                return Ok(ExecutionResult::Stop(stopping_reason))
            }
            ExecutionResult::Continue(continuation) => match continuation {
                PicoValue::String(string_value) => {
                    let match_result = self.regmatch.0.is_match(&string_value);

                    debug!(
                        "Regmatch: {:?} / {:?} = {:?}",
                        self.regmatch, string_value, match_result
                    );

                    let re_captures = self.regmatch.0.captures(&string_value);
                    debug!("LOC {:?}", re_captures);

                    return Ok(ExecutionResult::Continue(PicoValue::Boolean(match_result)));
                }
                _ => return Err(PicoError::IncompatibleComparison),
            },
            c => return Ok(c),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartsWith {
    match_start: (ValueProducer, ValueProducer), // needle, haystack
}
impl Execution for StartsWith {
    fn name(&self) -> String {
        return "startswith".to_string();
    }
    fn run_with_context(&self, state: &mut PicoState, ctx: &mut PicoContext) -> FnResult {
        let needle_ctx = self.match_start.0.run_with_context(state, ctx)?;
        let haystack_ctx = self.match_start.1.run_with_context(state, ctx)?;

        match (needle_ctx, haystack_ctx) {
            (
                ExecutionResult::Continue(needle_continuation),
                ExecutionResult::Continue(haystack_continuation),
            ) => {
                match (needle_continuation, haystack_continuation) {
                    (PicoValue::String(needle), PicoValue::String(haystack)) => {
                        // do stuff
                        let needle_str = needle.as_str();
                        let haystack_str = haystack.as_str();

                        let b = haystack_str.starts_with(needle_str);
                        return Ok(ExecutionResult::Continue(PicoValue::Boolean(b)));
                    }
                    _ => return Err(PicoError::IncompatibleComparison),
                }
            }
            _ => return Ok(ExecutionResult::Stop(Some("Stopping".to_string()))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    r#match: (ValueProducer, ValueProducer),
}
impl Execution for Match {
    fn name(&self) -> String {
        return "match".to_string();
    }

    fn run_with_context(&self, state: &mut PicoState, ctx: &mut PicoContext) -> FnResult {
        info!("running match");
        let lhs = self.r#match.0.run_with_context(state, ctx)?;
        let rhs = self.r#match.1.run_with_context(state, ctx)?;

        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                match (left, right) {
                    (PicoValue::String(ls), PicoValue::String(rs)) => {
                        let re = Regex::new(&rs).unwrap();
                        let b = re.is_match(&ls);
                        return Ok(ExecutionResult::Continue(PicoValue::Boolean(b)));
                    }
                    _ => return Err(PicoError::IncompatibleComparison),
                }
            }
            _ => {
                return Ok(ExecutionResult::Stop(Some(
                    "match requested stop".to_string(),
                )))
            }
        }
    }
}
