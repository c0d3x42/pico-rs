pub mod action;
pub mod execution;
pub mod flow_control;
pub mod logging;
pub mod setting;

use serde::{Deserialize, Serialize};

use crate::commands::execution::{ActionExecution, ActionResult, ActionValue};
use crate::commands::flow_control::{BreakToCommand, IfThenElse, MifThenElse, StopCommand};
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
    MifThenElse(Box<MifThenElse>),
    IfThenElse(Box<IfThenElse>),
    BreakTo(BreakToCommand),
    Stop(StopCommand),
    Set(SetCommand),
}
impl ActionExecution for Command {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ActionResult {
        info!("Running command...");
        match self {
            Command::MifThenElse(mif) => mif.run_with_context(pico_rules, runtime, ctx),
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
impl ActionExecution for PopLocals {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ActionResult {
        let hm = ctx.local_pop();

        Ok(ActionValue::Setting(hm))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FiniCommand {
    Log(Log),
    DebugLog(DebugLog),
    PopLocals(PopLocals),
}
impl ActionExecution for FiniCommand {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ActionResult {
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
