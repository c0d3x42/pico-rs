use serde::{Deserialize, Serialize, Serializer};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleFile {
    root: Vec<IfThenElse>,
    version: Option<String>,
}
