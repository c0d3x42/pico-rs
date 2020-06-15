use serde::{Deserialize, Serialize};

use crate::conditions::Condition;
use crate::context::{Context, VariablesMap};
use crate::errors::PicoError;
use crate::values::PicoValue;
//use crate::PicoValue;

use std::result;
use tinytemplate::TinyTemplate;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum ExecutionResult {
    Continue(PicoValue),
    Stop(Option<String>),
    BreakTo(uuid::Uuid),
}

pub type MyResult<T> = result::Result<T, PicoError>;
pub type FnResult = MyResult<ExecutionResult>;

// pub type FnResult = Result<ExecutionResult, PicoError>;

pub trait Execution {
    fn name(&self) -> String;
    fn alises(&self) -> Vec<String> {
        vec![]
    }
    fn run(&self) -> FnResult {
        Err(PicoError::Crash("Not implemented".to_string()))
        /*
        Err(ErrorResult::Crash(
            format!("Not done for: {}", &self.name()).to_string(),
        ))
        */
    }

    fn run_with_context(&self, _ctx: &mut Context) -> FnResult {
        trace!("Running with context for: {}", &self.name());
        Err(PicoError::Crash("Not implemented".to_string()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    log: String,
}
impl Execution for Log {
    fn name(&self) -> String {
        return "log".to_string();
    }
    fn run_with_context(&self, _ctx: &mut Context) -> FnResult {
        info!("MSG: {:?}", self.log);

        return Ok(ExecutionResult::Continue(PicoValue::Boolean(true)));
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DebugLog {
    debug: String,

    #[serde(default = "DebugLog::default_tt", skip)]
    tt: String,
}
impl DebugLog {
    fn default_tt() -> String {
        return "TTT".to_string();
    }
}
impl Execution for DebugLog {
    fn name(&self) -> String {
        return "debug-log".to_string();
    }
    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        let mut tt = TinyTemplate::new();
        trace!("Building tiny template");

        match tt.add_template("debug", &self.debug) {
            Err(e) => {
                error!("template failure: {:?}", e);
                return Err(PicoError::Crash("Template failure".to_string()));
            }
            Ok(_) => {}
        }

        let rendered = tt.render("debug", &ctx.variables);
        trace!("MSG: {:?}, variables: {:#?}", self.debug, &ctx.variables);

        match rendered {
            Ok(val) => debug!("tmpl[{:?}]: {:?}", self.tt, val),
            Err(e) => error!("{:?}", e),
        }

        return Ok(ExecutionResult::Continue(PicoValue::Boolean(true)));
    }
}

enum SettableValue {
    String,
    Number,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetCommand {
    set: (String, PicoValue),
}
impl Execution for SetCommand {
    fn name(&self) -> String {
        return "Set Command".to_string();
    }
    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        info!("RUNNING SET");

        match &self.set.1 {
            PicoValue::String(val) => {
                let c = PicoValue::String(val.to_string());
                ctx.local_variables.insert(self.set.0.to_string(), c);
            }
            PicoValue::Number(val) => {
                let c = PicoValue::Number(*val);
                ctx.local_variables.insert(self.set.0.to_string(), c);
            }
            PicoValue::UnsignedNumber(val) => {
                let c = PicoValue::UnsignedNumber(*val);
                ctx.local_variables.insert(self.set.0.to_string(), c);
            }
            something_else => {
                info!("SOMETHING else {:?}", something_else);
                ctx.local_variables
                    .insert(self.set.0.to_string(), something_else.clone());
            }
        }
        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}

/*
enum_str!(CommandWord{
    Stop("stop") // https://stackoverflow.com/questions/35134684/deserialize-to-struct-with-an-enum-member
});
*/

#[derive(Serialize, Deserialize, Debug)]
pub struct StopCommand {
    stop: String,
}
impl Execution for StopCommand {
    fn name(&self) -> String {
        return "Stop Command".to_string();
    }
    fn run_with_context(&self, _ctx: &mut Context) -> FnResult {
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
    fn run_with_context(&self, _ctx: &mut Context) -> FnResult {
        debug!("breaking to {:?}", self.r#break);
        Ok(ExecutionResult::BreakTo(self.r#break))
    }
}

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
        return "Command".to_string();
    }
    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        info!("Running command...");
        match self {
            Command::IfThenElse(ite) => ite.run_with_context(ctx),
            Command::Log(log) => log.run_with_context(ctx),
            Command::DebugLog(debug_log) => debug_log.run_with_context(ctx),
            Command::BreakTo(bto) => bto.run_with_context(ctx),
            Command::Stop(sto) => sto.run_with_context(ctx),
            Command::Set(se) => se.run_with_context(ctx),
        }
    }
}

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
    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        return match self {
            Action::Command(command) => command.run_with_context(ctx),
            Action::Commands(commands) => {
                for command in commands {
                    debug!("Running a command {:?}", command);
                    let result = command.run_with_context(ctx)?;
                    debug!("result: {:?}", result);
                    match result {
                        ExecutionResult::Stop(stopping_reason) => {
                            info!("Action collection terminated {:?}", stopping_reason);
                            return Ok(ExecutionResult::Stop(stopping_reason));
                            //return Ok(ExecutionResult::Continue(Value::Boolean(true)));
                        }
                        ExecutionResult::Continue(_value) => {}
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
        return Uuid::new_v4();
    }
}

impl Execution for IfThenElse {
    fn name(&self) -> String {
        let s = format!("ifthenelse [{:?}]", self.uuid);
        return s;
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        info!("running ITE -> {:?}", self.uuid);
        let if_result = self.r#if.run_with_context(ctx)?;
        match if_result {
            ExecutionResult::BreakTo(bto) => return Ok(ExecutionResult::BreakTo(bto)),
            ExecutionResult::Stop(stp) => return Ok(ExecutionResult::Stop(stp)),
            ExecutionResult::Continue(opt) => match opt {
                PicoValue::Boolean(b) => {
                    debug!("ITE got boolean back {:?}", b);
                    let branch_result = match b {
                        true => self.then.run_with_context(ctx),
                        false => match &self.r#else {
                            None => Ok(ExecutionResult::Continue(PicoValue::Boolean(true))),
                            Some(else_branch) => else_branch.run_with_context(ctx),
                        },
                    };
                    // then OR else has run, check the result
                    match branch_result {
                        Err(unhappy) => return Err(unhappy),
                        Ok(happy_result) => match happy_result {
                            ExecutionResult::BreakTo(bto_uuid) => {
                                debug!("Checking breakto {:?} == {:?}", self.uuid, bto_uuid);
                                if bto_uuid == self.uuid {
                                    debug!("breakto stopping");
                                    return Ok(ExecutionResult::Stop(None));
                                }
                                return Ok(ExecutionResult::BreakTo(bto_uuid));
                            }
                            c => return Ok(c), // passback everything else as is
                        },
                    }
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

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleFile {
    pub root: Vec<IfThenElse>,
    version: Option<String>,
}

impl Execution for RuleFile {
    fn name(&self) -> String {
        return "rule-file".to_string();
    }
}

#[test]
fn has_name() {
    assert_eq!(2 + 2, 4);
}
