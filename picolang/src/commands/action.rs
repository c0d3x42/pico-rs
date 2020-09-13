use serde::{Deserialize, Serialize};

use crate::commands::execution::{ActionExecution, ActionResult, ActionValue};
use crate::commands::Command;
use crate::context::PicoContext;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Action {
    Command(Command),
    Commands(Vec<Command>),
}
impl ActionExecution for Action {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ActionResult {
        match self {
            Action::Command(command) => command.run_with_context(pico_rules, runtime, ctx),
            Action::Commands(commands) => {
                for command in commands {
                    debug!("Running a command {:?}", command);
                    let result = command.run_with_context(pico_rules, runtime, ctx)?;
                    debug!("result: {:?}", result);
                    match result {
                        ActionValue::Stop(stopping_reason) => {
                            info!("Action collection terminated {:?}", stopping_reason);
                            return Ok(ActionValue::Stop(stopping_reason));
                        }
                        ActionValue::Continue => {}
                        ActionValue::Setting(_value) => {}
                        ActionValue::BreakTo(breakto) => {
                            info!("result breaks to {:?}", breakto);
                            return Ok(ActionValue::BreakTo(breakto));
                        }
                    }
                }
                Ok(ActionValue::Continue)
            }
        }
    }
}
