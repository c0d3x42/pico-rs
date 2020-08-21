pub mod action;
pub mod execution;
pub mod flow_control;
pub mod logging;
pub mod setting;

use serde::{Deserialize, Serialize};

use crate::commands::execution::{Execution, FnResult};
use crate::commands::flow_control::{BreakToCommand, IfThenElse, StopCommand};
use crate::commands::logging::{DebugLog, Log};
use crate::commands::setting::SetCommand;
use crate::context::PicoContext;
//use crate::state::PicoState;
use crate::loader::PicoRules;
use crate::loader::PicoRuntime as PicoState;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Command {
    Log(Log),
    DebugLog(DebugLog),
    IfThenElse(Box<IfThenElse>),
    BreakTo(BreakToCommand),
    Stop(StopCommand),
    Set(SetCommand),
}
impl Execution for Command {
    fn name(&self) -> String {
        String::from("Command")
    }
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        state: &mut PicoState,
        ctx: &mut PicoContext,
    ) -> FnResult {
        info!("Running command...");
        match self {
            Command::IfThenElse(ite) => ite.run_with_context(pico_rules, state, ctx),
            Command::Log(log) => log.run_with_context(pico_rules, state, ctx),
            Command::DebugLog(debug_log) => debug_log.run_with_context(pico_rules, state, ctx),
            Command::BreakTo(bto) => bto.run_with_context(pico_rules, state, ctx),
            Command::Stop(sto) => sto.run_with_context(pico_rules, state, ctx),
            Command::Set(se) => se.run_with_context(pico_rules, state, ctx),
        }
    }
}
