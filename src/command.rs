use serde::{Deserialize, Serialize, Serializer};

use crate::context::pico::{Context, VariablesMap};
use regex::Regex;
use serde_regex;
use std::error::Error;
use std::fmt;
use std::result;
use tinytemplate::TinyTemplate;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum ExecutionResult {
    Continue(Value),
    Stop(Option<String>),
}

#[derive(Debug)]
pub enum PicoError {
    IncompatibleComparison,
    NoSuchValue,
    Crash(String),
}

impl fmt::Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {:?}", self)
    }
}
impl Error for PicoError {}

type MyResult<T> = result::Result<T, PicoError>;
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

    fn run_with_context(&self, _variables: &VariablesMap) -> FnResult {
        trace!("Running with context for: {}", &self.name());
        Err(PicoError::Crash("Not implemented".to_string()))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Value {
    UnsignedNumber(usize),
    Number(isize),
    String(String),
    Boolean(bool),
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Boolean(a), &Value::Boolean(b)) => a == b,
            (&Value::UnsignedNumber(a), &Value::UnsignedNumber(b)) => a == b,
            (&Value::Number(a), &Value::Number(b)) => a == b,
            (&Value::String(ref a), &Value::String(ref b)) => a == b,
            _ => false,
        }
    }
}

#[test]
fn eq9() {
    let v = Value::Number(9);
    assert_eq!(v.eq(&Value::Number(9)), true);
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum VarValue {
    Lookup(String),
    DefaultLookup(String, Value),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarLookup {
    var: VarValue,
}

impl Execution for VarLookup {
    fn name(&self) -> String {
        return "VarLookup".to_string();
    }

    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        match &self.var {
            // Plain lookup in ctx variables
            VarValue::Lookup(s) => {
                debug!("lookup {:?}", s);
                let lookup = variables.get(s);
                match lookup {
                    Some(v) => {
                        let r = v.clone();
                        return Ok(ExecutionResult::Continue(r));
                    }
                    None => {
                        return Ok(ExecutionResult::Stop(Some(
                            format!("No such value {:?}", s).to_string(),
                        )))
                    }
                };
            }
            VarValue::DefaultLookup(varname, fallback) => {
                debug!("default lookup {:?}, {:?}", varname, fallback);

                let lookup = variables.get(varname);
                match lookup {
                    Some(value) => return Ok(ExecutionResult::Continue(value.clone())),
                    None => return Ok(ExecutionResult::Continue(fallback.clone())),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Var {
    Literal(Value),
    Lookup(VarLookup),
}

impl Execution for Var {
    fn name(&self) -> String {
        return "Var".to_string();
    }

    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        match self {
            Var::Lookup(lookup) => lookup.run_with_context(variables),
            Var::Literal(literal) => Ok(ExecutionResult::Continue(literal.clone())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct And {
    and: Vec<Condition>,
}

impl Execution for And {
    fn name(&self) -> String {
        return "and".to_string();
    }
    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        for condition in &self.and {
            let condition_result = condition.run_with_context(variables)?;

            match condition_result {
                ExecutionResult::Stop(stopping_reason) => {
                    return Ok(ExecutionResult::Stop(stopping_reason))
                }
                ExecutionResult::Continue(continuation) => match continuation {
                    Value::Boolean(b) => {
                        if !b {
                            // AND exits as soon as one condition returns boolean false
                            return Ok(ExecutionResult::Continue(Value::Boolean(false)));
                        }
                    }
                    _ => return Err(PicoError::Crash("non boolean".to_string())),
                },
            }
        }
        // all conditions returned boolean true
        Ok(ExecutionResult::Continue(Value::Boolean(true)))
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

    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        let condition_count = self.or.len();
        debug!("OR ...{:?}", condition_count);

        for condition in &self.or {
            let condition_result = condition.run_with_context(variables)?;

            match condition_result {
                ExecutionResult::Stop(stopping) => return Ok(ExecutionResult::Stop(stopping)),
                ExecutionResult::Continue(continuation) => match continuation {
                    Value::Boolean(b) => {
                        if b {
                            // OR completes succesfully on the first boolean true
                            return Ok(ExecutionResult::Continue(Value::Boolean(true)));
                        }
                    }
                    _ => return Err(PicoError::Crash("Non boolean".to_string())),
                },
            }
        }
        Ok(ExecutionResult::Continue(Value::Boolean(false)))
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
    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        let lhs = self.eq.0.run_with_context(variables)?;
        let rhs = self.eq.1.run_with_context(variables)?;

        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                return Ok(ExecutionResult::Continue(Value::Boolean(left == right)))
            }

            _ => return Ok(ExecutionResult::Continue(Value::Boolean(false))),
        }
    }
}

#[test]
fn var1var1() {
    let vm = VariablesMap::new();
    let var1 = Var::Literal(Value::String("q".to_string()));
    let var2 = Var::Literal(Value::String("q".to_string()));
    let var3 = Var::Literal(Value::String("xnot".to_string()));
    let var4 = Var::Literal(Value::String("not".to_string()));
    let eq1 = Eq { eq: (var1, var2) };
    let t = eq1.run_with_context(&vm);
    assert_eq!(t.is_ok(), true);

    let eq2 = Eq { eq: (var3, var4) };
    let x = eq2.run_with_context(&vm);
    assert!(x.is_ok());
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

    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        debug!("Looking up regmatch/with");

        let with_value = self.with.run_with_context(variables)?;

        match with_value {
            ExecutionResult::Stop(stopping_reason) => {
                return Ok(ExecutionResult::Stop(stopping_reason))
            }
            ExecutionResult::Continue(continuation) => match continuation {
                Value::String(string_value) => {
                    let match_result = self.regmatch.is_match(&string_value);

                    debug!(
                        "Regmatch: {:?} / {:?} = {:?}",
                        self.regmatch, string_value, match_result
                    );

                    let loc = self.regmatch.captures(&string_value);
                    debug!("LOC {:?}", loc);

                    return Ok(ExecutionResult::Continue(Value::Boolean(match_result)));
                }
                _ => return Err(PicoError::IncompatibleComparison),
            },
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
    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        let needle_ctx = self.match_start.0.run_with_context(variables)?;
        let haystack_ctx = self.match_start.1.run_with_context(variables)?;

        match (needle_ctx, haystack_ctx) {
            (
                ExecutionResult::Continue(needle_continuation),
                ExecutionResult::Continue(haystack_continuation),
            ) => {
                match (needle_continuation, haystack_continuation) {
                    (Value::String(needle), Value::String(haystack)) => {
                        // do stuff
                        let needle_str = needle.as_str();
                        let haystack_str = haystack.as_str();

                        let b = haystack_str.starts_with(needle_str);
                        return Ok(ExecutionResult::Continue(Value::Boolean(b)));
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

    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        info!("running match");
        let lhs = self.r#match.0.run_with_context(variables)?;
        let rhs = self.r#match.1.run_with_context(variables)?;

        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                match (left, right) {
                    (Value::String(ls), Value::String(rs)) => {
                        let re = Regex::new(&rs).unwrap();
                        let b = re.is_match(&ls);
                        return Ok(ExecutionResult::Continue(Value::Boolean(b)));
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

    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        let condition_result = self.not.run_with_context(variables)?;

        match condition_result {
            ExecutionResult::Continue(val) => match val {
                Value::Boolean(b) => {
                    return Ok(ExecutionResult::Continue(Value::Boolean(!b)));
                }
                _ => return Err(PicoError::IncompatibleComparison),
            },
            ExecutionResult::Stop(s) => return Ok(ExecutionResult::Stop(s)),
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
    Not(Not),
}

impl Execution for Condition {
    fn name(&self) -> String {
        "condition".to_string()
    }

    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        debug!("Checking condition {:?}", self);
        match self {
            Condition::And(and) => and.run_with_context(variables),
            Condition::Or(or) => or.run_with_context(variables),
            Condition::Not(not) => not.run_with_context(variables),
            Condition::Match(m) => m.run_with_context(variables),
            Condition::RegMatch(rm) => rm.run_with_context(variables),
            Condition::StartsWith(sw) => sw.run_with_context(variables),

            Condition::Eq(eq) => eq.run_with_context(variables),

            _ => Err(PicoError::Crash("no such condition".to_string())),
        }
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
    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        info!("MSG: {:?}", self.log);

        return Ok(ExecutionResult::Continue(Value::Boolean(true)));
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
    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        let mut tt = TinyTemplate::new();
        trace!("Building tiny template");

        match tt.add_template("debug", &self.debug) {
            Err(e) => {
                error!("template failure: {:?}", e);
                return Err(PicoError::Crash("Template failure".to_string()));
            }
            Ok(_) => {}
        }

        let rendered = tt.render("debug", variables);
        trace!("MSG: {:?}, variables: {:#?}", self.debug, variables);

        match rendered {
            Ok(val) => debug!("tmpl[{:?}]: {:?}", self.tt, val),
            Err(e) => error!("{:?}", e),
        }

        return Ok(ExecutionResult::Continue(Value::Boolean(true)));
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Command {
    Log(Log),
    DebugLog(DebugLog),
    IfThenElse(Box<IfThenElse>),
}
impl Execution for Command {
    fn name(&self) -> String {
        return "Command".to_string();
    }
    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        info!("Running command...");
        match self {
            Command::IfThenElse(ite) => ite.run_with_context(variables),
            Command::Log(log) => log.run_with_context(variables),
            Command::DebugLog(debug_log) => debug_log.run_with_context(variables),
            (_) => Err(PicoError::Crash("command not impl".to_string())),
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
    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        return match self {
            Action::Command(command) => command.run_with_context(variables),
            Action::Commands(commands) => {
                for command in commands {
                    debug!("Running a command");
                    let result = command.run_with_context(variables)?;
                    match result {
                        ExecutionResult::Stop(stopping_reason) => {
                            info!("Action collection terminated {:?}", stopping_reason);
                            //return Ok(ExecutionResult::Continue(Value::Boolean(true)));
                        }
                        ExecutionResult::Continue(_value) => {}
                    }
                }
                return Ok(ExecutionResult::Continue(Value::Boolean(true)));
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

    fn run_with_context(&self, variables: &VariablesMap) -> FnResult {
        info!("running ITE -> {:?}", self.uuid);
        let if_result = self.r#if.run_with_context(variables)?;
        match if_result {
            ExecutionResult::Stop(stp) => return Ok(ExecutionResult::Stop(stp)),
            ExecutionResult::Continue(opt) => match opt {
                Value::Boolean(b) => {
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
                }
                _ => return Ok(ExecutionResult::Stop(None)),
            },
        }
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
