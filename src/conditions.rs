use serde::{Deserialize, Serialize};

use crate::command::{Execution, ExecutionResult, FnResult};
use crate::context::{Context, VariablesMap};
use crate::errors::PicoError;
//use crate::values::{PicoValue, Var};
use crate::{PicoValue, Var};

use regex::Regex;
use serde_regex;

/*
 * existance or not of variables in the context
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct VarExistsCondition {
    exists: String,
}
impl Execution for VarExistsCondition {
    fn name(&self) -> String {
        return "VarExists".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        let variables = &ctx.variables;
        let t = variables.contains_key(&self.exists);
        return Ok(ExecutionResult::Continue(PicoValue::Boolean(t)));
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarMissingCondition {
    missing: String,
}
impl Execution for VarMissingCondition {
    fn name(&self) -> String {
        return "VarMissing".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        let t = ctx.variables.contains_key(&self.missing);
        return Ok(ExecutionResult::Continue(PicoValue::Boolean(!t)));
    }
}

/*
 * condition collections
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct And {
    and: Vec<Condition>,
}

impl Execution for And {
    fn name(&self) -> String {
        return "and".to_string();
    }
    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        for condition in &self.and {
            let condition_result = condition.run_with_context(ctx)?;

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
        return "or".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        let condition_count = self.or.len();
        debug!("OR ...{:?}", condition_count);

        for condition in &self.or {
            let condition_result = condition.run_with_context(ctx)?;

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
pub struct Eq {
    eq: (Var, Var),
}
impl Execution for Eq {
    fn name(&self) -> String {
        return "equality".to_string();
    }
    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        let lhs = self.eq.0.run_with_context(ctx)?;
        let rhs = self.eq.1.run_with_context(ctx)?;

        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                return Ok(ExecutionResult::Continue(PicoValue::Boolean(left == right)))
            }

            _ => return Ok(ExecutionResult::Continue(PicoValue::Boolean(false))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LessThan {
    lt: (Var, Var),
}
impl Execution for LessThan {
    fn name(&self) -> String {
        return "less than".to_string();
    }
    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        let lhs = self.lt.0.run_with_context(ctx)?;
        let rhs = self.lt.1.run_with_context(ctx)?;
        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                Ok(ExecutionResult::Continue(PicoValue::Boolean(left < right)))
            }
            _ => Ok(ExecutionResult::Continue(PicoValue::Boolean(false))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegMatch {
    #[serde(with = "serde_regex")]
    regmatch: Regex,

    with: Var,
}

impl Execution for RegMatch {
    fn name(&self) -> String {
        return "regmatch".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        debug!("Looking up regmatch/with");

        let with_value = self.with.run_with_context(ctx)?;

        match with_value {
            ExecutionResult::Stop(stopping_reason) => {
                return Ok(ExecutionResult::Stop(stopping_reason))
            }
            ExecutionResult::Continue(continuation) => match continuation {
                PicoValue::String(string_value) => {
                    let match_result = self.regmatch.is_match(&string_value);

                    debug!(
                        "Regmatch: {:?} / {:?} = {:?}",
                        self.regmatch, string_value, match_result
                    );

                    let loc = self.regmatch.captures(&string_value);
                    debug!("LOC {:?}", loc);

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
    match_start: (Var, Var), // needle, haystack
}
impl Execution for StartsWith {
    fn name(&self) -> String {
        return "startswith".to_string();
    }
    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        let needle_ctx = self.match_start.0.run_with_context(ctx)?;
        let haystack_ctx = self.match_start.1.run_with_context(ctx)?;

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
    r#match: (Var, Var),
}
impl Execution for Match {
    fn name(&self) -> String {
        return "match".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        info!("running match");
        let lhs = self.r#match.0.run_with_context(ctx)?;
        let rhs = self.r#match.1.run_with_context(ctx)?;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Not {
    not: Box<Condition>,
}
impl Execution for Not {
    fn name(&self) -> String {
        return "not".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        let condition_result = self.not.run_with_context(ctx)?;

        match condition_result {
            ExecutionResult::Continue(val) => match val {
                PicoValue::Boolean(b) => {
                    return Ok(ExecutionResult::Continue(PicoValue::Boolean(!b)));
                }
                _ => return Err(PicoError::IncompatibleComparison),
            },
            c => return Ok(c), //ExecutionResult::Stop(s) => return Ok(ExecutionResult::Stop(s)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Condition {
    And(And),
    Or(Or),
    Eq(Eq),
    Match(Match),
    RegMatch(RegMatch),
    StartsWith(StartsWith),
    LessThan(LessThan),
    VarExists(VarExistsCondition),
    VarMissing(VarMissingCondition),
    Not(Not),
}

impl Execution for Condition {
    fn name(&self) -> String {
        "condition".to_string()
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        debug!("Checking condition {:?}", self);
        let condition_result = match self {
            Condition::And(and) => and.run_with_context(ctx),
            Condition::Or(or) => or.run_with_context(ctx),
            Condition::Not(not) => not.run_with_context(ctx),
            Condition::Match(m) => m.run_with_context(ctx),
            Condition::RegMatch(rm) => rm.run_with_context(ctx),
            Condition::StartsWith(sw) => sw.run_with_context(ctx),

            Condition::Eq(eq) => eq.run_with_context(ctx),
            Condition::LessThan(lt) => lt.run_with_context(ctx),

            Condition::VarExists(ve) => ve.run_with_context(ctx),
            Condition::VarMissing(vm) => vm.run_with_context(ctx),
        };

        match condition_result {
            Ok(result) => Ok(result),
            Err(error_result) => match error_result {
                PicoError::NoSuchValue | PicoError::IncompatibleComparison => {
                    info!("condition result was bad - mapping to false");
                    return Ok(ExecutionResult::Continue(PicoValue::Boolean(false)));
                }
                err => Err(err),
            },
        }
    }
}
