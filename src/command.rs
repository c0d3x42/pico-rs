use serde::{Deserialize, Serialize, Serializer};

use crate::context::pico::{Context, VariablesMap};
use tinytemplate::TinyTemplate;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Value {
    UnsignedNumber(usize),
    Number(isize),
    String(String),
    Boolean(bool),
}
impl Value {
    pub fn cmp_match(&self, other: &Value) -> ExecutionResult {
        match (self, other) {
            (Value::String(s1), Value::String(s2)) => {
                return ExecutionResult::Continue(Some(Value::Boolean(s1 == s2)));
            }
            _ => return ExecutionResult::Error("mismatched comparisions".to_string()),
        }
    }
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

impl Execution for VarLookup {
    fn name(&self) -> String {
        return "VarLookup".to_string();
    }

    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        match &self.var {
            VarValue::Lookup(s) => {
                debug!("lookup {:?}", s);
                let lookup = variables.get(s);
                return match lookup {
                    Some(v) => {
                        let r = v.clone();
                        return ExecutionResult::Continue(Some(r));
                    }
                    None => ExecutionResult::Continue(None),
                };
            }
            VarValue::DefaultLookup(s, ov) => {
                debug!("default lookup {:?}, {:?}", s, ov);

                let lookup = variables.get(s);
                match lookup {
                    Some(v) => return ExecutionResult::Continue(Some(v.clone())),
                    None => match ov {
                        Some(fallback) => return ExecutionResult::Continue(Some(fallback.clone())),
                        _ => {}
                    },
                }
                info!("no default lookup");

                return ExecutionResult::Continue(None);
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

    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        match self {
            Var::Lookup(lookup) => lookup.run_with_context(variables),
            Var::Literal(literal) => ExecutionResult::Continue(Some(literal.clone())),
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
    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        for condition in &self.and {
            match condition.run_with_context(variables) {
                ExecutionResult::Continue(cont) => match cont {
                    None => break,
                    Some(s) => match s {
                        Value::Boolean(b) => {
                            if !b {
                                break;
                            };
                        }
                        _ => {}
                    },
                },
                bad => return bad,
            }
        }
        ExecutionResult::Continue(None)
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

    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        let condition_count = self.or.len();
        debug!("OR ...{:?}", condition_count);

        for condition in &self.or {
            match condition.run_with_context(variables) {
                ExecutionResult::Continue(cont) => match cont {
                    None => break,
                    Some(condition_result) => match condition_result {
                        Value::Boolean(b) => {
                            if b {
                                // finished with the first condition to return true
                                return ExecutionResult::Continue(Some(Value::Boolean(true)));
                            }
                        }
                        _ => {
                            debug!("OR condition execution did not return a bool");
                        }
                    },
                },
                failure => return failure,
            }
        }
        // no conditions returned true

        ExecutionResult::Continue(Some(Value::Boolean(false)))
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
    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        let lhs = self.eq.0.run_with_context(variables);
        let rhs = self.eq.1.run_with_context(variables);

        trace!("EQ lhs: {:?}, rhs: {:?}", lhs, rhs);

        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                match (left, right) {
                    (Some(l), Some(r)) => {
                        return ExecutionResult::Continue(Some(Value::Boolean(l == r)))
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        return ExecutionResult::Continue(None);
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

    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        info!("running match");
        let lhs = self.r#match.0.run_with_context(variables);
        let rhs = self.r#match.1.run_with_context(variables);

        match (lhs, rhs) {
            (ExecutionResult::Continue(left), ExecutionResult::Continue(right)) => {
                match (left, right) {
                    (Some(l), Some(r)) => {
                        info!("L: {:?}, R: {:?}", l, r);
                        let b = l == r;
                        return l.cmp_match(&r);
                        // return ExecutionResult::Continue(None);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        debug!("No match");

        return ExecutionResult::Continue(Some(Value::Boolean(false)));
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

    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        match self.not.run_with_context(variables) {
            ExecutionResult::Continue(cont) => match cont {
                None => ExecutionResult::Continue(None),
                Some(condition_result) => match condition_result {
                    Value::Boolean(b) => ExecutionResult::Continue(Some(Value::Boolean(!b))),
                    _ => ExecutionResult::Continue(None),
                },
            },
            bad => return bad,
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
    Not(Not),
}

impl Execution for Condition {
    fn name(&self) -> String {
        "condition".to_string()
    }

    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        match self {
            Condition::And(and) => and.run_with_context(variables),
            Condition::Or(or) => or.run_with_context(variables),
            Condition::Not(not) => not.run_with_context(variables),
            Condition::Match(m) => m.run_with_context(variables),
            Condition::Eq(eq) => eq.run_with_context(variables),
            (_) => ExecutionResult::Error("not impl".to_string()),
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
    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        info!("MSG: {:?}", self.log);

        return ExecutionResult::Continue(None);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DebugLog {
    debug: String,
}
impl Execution for DebugLog {
    fn name(&self) -> String {
        return "debug-log".to_string();
    }
    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        let mut tt = TinyTemplate::new();
        tt.add_template("debug", &self.debug);

        let rendered = tt.render("debug", variables);
        trace!("MSG: {:?}, variables: {:#?}", self.debug, variables);

        match rendered {
            Ok(val) => debug!("tmpl: {:?}", val),
            Err(e) => error!("{:?}", e),
        }

        return ExecutionResult::Continue(None);
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
    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        info!("Running command...");
        match self {
            Command::IfThenElse(ite) => ite.run_with_context(variables),
            Command::Log(log) => log.run_with_context(variables),
            Command::DebugLog(debug_log) => debug_log.run_with_context(variables),
            (_) => ExecutionResult::Error("command not impl".to_string()),
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
    fn run_with_context(&self, variables: &VariablesMap) -> ExecutionResult {
        match self {
            Action::Command(command) => command.run_with_context(variables),
            Action::Commands(commands) => {
                for command in commands {
                    debug!("Running a command");
                    command.run_with_context(variables);
                }
                return ExecutionResult::Continue(Some(Value::Boolean(true)));
            }
            (_) => ExecutionResult::Error("unhandled action".to_string()),
        }
    }
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
        info!("running ITE");
        let if_result = self.r#if.run_with_context(variables);
        match if_result {
            ExecutionResult::Continue(opt) => match opt {
                Some(optional) => match optional {
                    Value::Boolean(b) => {
                        if (b) {
                            info!("ITE: then branch");
                            return self.then.run_with_context(variables);
                        } else {
                            info!("ITE: else branch");
                            return self.r#else.run_with_context(variables);
                        }
                    }
                    (_) => return ExecutionResult::Error("if result unexpected".to_string()),
                },
                None => return ExecutionResult::Continue(None),
            },
            (_) => {}
        }
        return if_result;
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
