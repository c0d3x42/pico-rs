pub mod action;
pub mod execution;
pub mod flow_control;
pub mod logging;
pub mod setting;

use serde::{Deserialize, Serialize};

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::commands::flow_control::{BreakToCommand, IfThenElse, StopCommand};
use crate::commands::logging::{DebugLog, Log};
use crate::commands::setting::SetCommand;
use crate::context::PicoContext;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;

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
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        info!("Running command...");
        match self {
            Command::IfThenElse(ite) => ite.run_with_context(pico_rules, runtime, ctx),
            Command::Log(log) => log.run_with_context(pico_rules, runtime, ctx),
            Command::DebugLog(debug_log) => debug_log.run_with_context(pico_rules, runtime, ctx),
            Command::BreakTo(bto) => bto.run_with_context(pico_rules, runtime, ctx),
            Command::Stop(sto) => sto.run_with_context(pico_rules, runtime, ctx),
            Command::Set(se) => se.run_with_context(pico_rules, runtime, ctx),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PopLocals {
    pop_locals: bool,
}
impl Execution for PopLocals {
    fn name(&self) -> String {
        String::from("Fini Command")
    }
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        let hm = runtime.json_pop();
        Ok(ExecutionResult::Setting(hm))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FiniCommand {
    Log(Log),
    DebugLog(DebugLog),
    PopLocals(PopLocals),
}
impl Execution for FiniCommand {
    fn name(&self) -> String {
        String::from("Fini Command")
    }
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        info!("Running finish command...");
        match self {
            FiniCommand::Log(log) => log.run_with_context(pico_rules, runtime, ctx),
            FiniCommand::DebugLog(debug_log) => {
                debug_log.run_with_context(pico_rules, runtime, ctx)
            }
            FiniCommand::PopLocals(se) => se.run_with_context(pico_rules, runtime, ctx),
        }
    }
}
