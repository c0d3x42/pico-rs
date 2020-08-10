use serde::{Deserialize, Serialize};

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::commands::Command;
use crate::context::PicoContext;
use crate::state::PicoState;
use crate::values::PicoValue;

//use std::result;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Action {
    Command(Command),
    Commands(Vec<Command>),
}
impl Execution for Action {
    fn name(&self) -> String {
        return "Action".to_string();
    }
    fn run_with_context(&self, state: &mut PicoState, ctx: &mut PicoContext) -> FnResult {
        return match self {
            Action::Command(command) => command.run_with_context(state, ctx),
            Action::Commands(commands) => {
                for command in commands {
                    debug!("Running a command {:?}", command);
                    let result = command.run_with_context(state, ctx)?;
                    debug!("result: {:?}", result);
                    match result {
                        ExecutionResult::Stop(stopping_reason) => {
                            info!("Action collection terminated {:?}", stopping_reason);
                            return Ok(ExecutionResult::Stop(stopping_reason));
                            //return Ok(ExecutionResult::Continue(Value::Boolean(true)));
                        }
                        ExecutionResult::Continue(_value) => {}
                        ExecutionResult::Setting(_value) => {}
                        ExecutionResult::BreakTo(breakto) => {
                            info!("result breaks to {:?}", breakto);
                            return Ok(ExecutionResult::BreakTo(breakto));
                        }
                    }
                }
                return Ok(ExecutionResult::Continue(PicoValue::Boolean(true)));
            }
        };
    }
}
