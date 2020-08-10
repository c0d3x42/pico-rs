use serde::{Deserialize, Serialize};

use crate::commands::action::Action;
use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::conditions::Condition;
use crate::context::PicoContext;
use crate::errors::PicoError;
use crate::state::PicoState;
use crate::values::PicoValue;

use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct StopCommand {
    stop: String,
}
impl Execution for StopCommand {
    fn name(&self) -> String {
        return "Stop Command".to_string();
    }
    fn run_with_context(&self, _state: &mut PicoState, _ctx: &mut PicoContext) -> FnResult {
        debug!("stopping because {:?}", self.stop);
        Ok(ExecutionResult::Stop(Some(self.stop.clone())))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BreakToCommand {
    r#break: uuid::Uuid,
}
impl Execution for BreakToCommand {
    fn name(&self) -> String {
        return "BreakTo Command".to_string();
    }
    fn run_with_context(&self, _state: &mut PicoState, _ctx: &mut PicoContext) -> FnResult {
        debug!("breaking to {:?}", self.r#break);
        Ok(ExecutionResult::BreakTo(self.r#break))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IfThenElse {
    r#if: Condition,
    r#then: Action,
    r#else: Option<Action>,

    #[serde(default = "IfThenElse::default_uuid")]
    uuid: uuid::Uuid,
}
impl IfThenElse {
    fn default_uuid() -> uuid::Uuid {
        trace!("assigning default uuid");
        return Uuid::new_v4();
    }
}

impl Execution for IfThenElse {
    fn name(&self) -> String {
        let s = format!("ifthenelse [{:?}]", self.uuid);
        return s;
    }

    fn run_with_context(&self, state: &mut PicoState, ctx: &mut PicoContext) -> FnResult {
        info!("running ITE -> {:?}", self.uuid);
        let if_result = self.r#if.run_with_context(state, ctx)?;
        state.increment_branch_hit(&self.uuid);
        match if_result {
            ExecutionResult::BreakTo(bto) => return Ok(ExecutionResult::BreakTo(bto)),
            ExecutionResult::Stop(stp) => return Ok(ExecutionResult::Stop(stp)),
            ExecutionResult::Setting(_dict) => {
                return Err(PicoError::Crash(String::from("cant set dict here")))
            }
            ExecutionResult::Continue(opt) => match opt {
                PicoValue::Boolean(b) => {
                    debug!("ITE got boolean back {:?}", b);

                    let branch_result = match b {
                        true => self.then.run_with_context(state, ctx),
                        false => match &self.r#else {
                            None => Ok(ExecutionResult::Continue(PicoValue::Boolean(true))),
                            Some(else_branch) => else_branch.run_with_context(state, ctx),
                        },
                    };
                    // then OR else has run, check the result
                    let command_result = match branch_result {
                        Err(unhappy) => Err(unhappy),
                        Ok(happy_result) => match happy_result {
                            ExecutionResult::BreakTo(bto_uuid) => {
                                debug!("Checking breakto {:?} == {:?}", self.uuid, bto_uuid);
                                if bto_uuid == self.uuid {
                                    debug!("breakto stopping");
                                    return Ok(ExecutionResult::Stop(None));
                                }
                                return Ok(ExecutionResult::BreakTo(bto_uuid));
                            }
                            c => Ok(c), // passback everything else as is
                        },
                    };

                    return command_result;
                    /*
                    if b {
                        info!("ITE: then branch");
                        return self.then.run_with_context(variables);
                    } else {
                        info!("ITE: else branch");
                        match &self.r#else {
                            None => {
                                debug!("else branch taken but nothing here");
                                return Ok(ExecutionResult::Continue(Value::Boolean(true)));
                            }
                            Some(else_branch) => return else_branch.run_with_context(variables),
                        }
                    }
                    */
                }
                _ => return Ok(ExecutionResult::Stop(None)),
            },
        };

        //return if_result;
    }
}