use serde::{Deserialize, Serialize, Serializer};

use crate::context::pico::{Context, VariablesMap};

#[derive(Clone, Debug)]
pub enum ExecutionResult {
    Continue(Option<Value>),
    Error(String),
    Crash(String),
    Exit(Option<String>),
}

pub trait Execution {
    fn name(&self) -> String;
    fn alises(&self) -> Vec<String> {
        vec![]
    }
    fn run(&self) -> ExecutionResult {
        ExecutionResult::Crash(format!("Not done for: {}", &self.name()).to_string())
    }

    fn run_with_context(&self, _variables: &VariablesMap) -> ExecutionResult {
        trace!("Running with context for: {}", &self.name());
        ExecutionResult::Crash(format!("Not done for: {}", &self.name()).to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Value {
    UnsignedNumber(usize),
    Number(isize),
    String(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum VarValue {
    Lookup(String),
    DefaultLookup(String, Option<Value>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarLookup {
    var: VarValue,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Var {
    Literal(Value),
    Lookup(VarLookup),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct And {
    and: Vec<Condition>,
}

impl Execution for And {
    fn name(&self) -> String {
        return "and".to_string();
    }
    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        for condition in &self.and {
            condition.run_with_context(variables);
        }
        ExecutionResult::Continue(None)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Or {
    or: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Eq {
    eq: (Var, Var),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    r#match: (Var, Var),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Not {
    not: Box<Condition>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Condition {
    And(And),
    Or(Or),
    Eq(Eq),
    Match(Match),
    Not(Not),
}

impl Execution for Condition {
    fn name(&self) -> String {
        "condition".to_string()
    }

    fn run_with_context(&self, _variables: &VariablesMap)->ExecutionResult{

    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    log: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DebugLog {
    debug: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Command {
    Log(Log),
    DebugLog(DebugLog),
    IfThenElse(Box<IfThenElse>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Action {
    Command(Command),
    Commands(Vec<Command>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IfThenElse {
    r#if: Condition,
    r#then: Action,
    r#else: Action,
}
impl Execution for IfThenElse {
    fn name(&self) -> String {
        return "ifthenelse".to_string();
    }

    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        let if_result = self.r#if.run_with_context(variables);
        match if_result {
            ExecutionResult::Continue(opt) => {
                if Some(tr) = opt //////
            }
        }
        return ExecutionResult::Error("xxx".to_string());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleFile {
    pub root: Vec<IfThenElse>,
    version: Option<String>,
}
