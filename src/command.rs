use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::conditions::Condition;
use crate::context::{Context, VariablesMap};
use crate::errors::PicoError;
use crate::values::{Extract, PicoValue, ValueProducer, Var};
//use crate::PicoValue;

use std::option;
use std::result;
use tinytemplate::TinyTemplate;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum ExecutionResult {
    Continue(PicoValue),
    Setting(HashMap<String, PicoValue>),
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

        // combine variables and local_variables into one hashmap for template rendering

        let c = ctx.variables.clone();
        let l = ctx.local_variables.clone();

        let k: HashMap<String, PicoValue> = l.into_iter().chain(c).collect();

        let rendered = tt.render("debug", &k);
        trace!("MSG: {:?}, variables: {:#?}", self.debug, &ctx.variables);

        match rendered {
            Ok(val) => debug!("tmpl[{:?}]: {:?}", self.tt, val),
            Err(e) => error!("{:?}", e),
        }

        return Ok(ExecutionResult::Continue(PicoValue::Boolean(true)));
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Settable {
    ValueProducing(String, ValueProducer),
    Extractor(Extract),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetCommand {
    set: Settable,
}
impl Execution for SetCommand {
    fn name(&self) -> String {
        return "Set Command".to_string();
    }
    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        info!("RUNNING SET");

        match &self.set {
            Settable::Extractor(extraction) => {
                let extracted_values = extraction.run_with_context(ctx)?;
                match extracted_values {
                    ExecutionResult::Setting(dict) => {
                        for (key, value) in dict {
                            ctx.setValue(&key, value);
                        }
                    }
                    _ => {}
                }
            }

            Settable::ValueProducing(var_name, value_producer) => {
                let produced_value = value_producer.run_with_context(ctx)?;

                debug!("Produced value = {:?}", produced_value);

                match produced_value {
                    ExecutionResult::Continue(pv) => match pv {
                        PicoValue::String(v) => {
                            ctx.setValue(var_name, PicoValue::String(v.to_string()))
                        }
                        PicoValue::Number(val) => ctx.setValue(var_name, PicoValue::Number(val)),
                        PicoValue::UnsignedNumber(val) => {
                            ctx.setValue(var_name, PicoValue::UnsignedNumber(val))
                        }
                        PicoValue::Boolean(val) => ctx.setValue(var_name, PicoValue::Boolean(val)),
                    },
                    _everything_else => {}
                }
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
            ExecutionResult::Setting(_dict) => {
                return Err(PicoError::Crash(String::from("cant set dict here")))
            }
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
