use crate::PicoValue;
use serde::Deserialize;
use std::collections::HashMap;

pub type LookupTableName = String;

///
/// The ondisk representation of a Pico rule file
#[derive(Serialize, Deserialize, Debug)]
pub struct RuleFile {
    pub version: String,

    #[serde(default)]
    pub lookups: HashMap<LookupTableName, LookupDefinition>,

    pub root: Vec<RuleInstruction>,
}

trait ExternalLookup {
    type Source;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum LookupDefinition {
    LookupTableInternal {
        entries: HashMap<String, PicoValue>,
        default: PicoValue,
    },
    LookupTableUrl(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Producer {
    If(IfOperation),
    Eq(EqOperation),
    Ne(NeOperation),
    Or(OrOperation),
    Lt(LessThanOperation),
    Var(VarOp),
    String(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RuleInstruction {
    Logic(IfOperation),
    Let(LetStmt),
    Set(SetStmt),
    Debug(DebugOperation),
}
/*
 * Logic operations
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct IfOperation {
    #[serde(rename = "if")]
    pub value: Vec<Producer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EqOperation {
    #[serde(rename = "==")]
    pub value: Box<(Producer, Producer)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NeOperation {
    #[serde(rename = "!=")]
    value: Box<(Producer, Producer)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrOperation {
    #[serde(rename = "or")]
    value: Vec<Producer>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AndOperation {
    #[serde(rename = "and")]
    value: Vec<Producer>,
}


/*
 * Numeric operations
 */

#[derive(Serialize, Deserialize, Debug)]
pub enum NumericOperation {
    LT(LessThanOperation),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LessThanOperation {
    #[serde(rename = "<")]
    /**
     * two or more producers
     */
    pub value: Vec<Producer>, 
}

/*
 * Arithmetic operations
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct AddOp {
    #[serde(rename = "+")]
    value: Vec<Producer>,
}

/*
 * Variable lookup
 */


#[derive(Serialize, Deserialize, Debug)]
pub enum VarType {
    Plain,
    Pointer,
    Path,
}
impl VarType {
    pub fn plain() -> Self {
        VarType::Plain
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleString (pub String);

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum VarValue {
    String(String),
    OneString([String;1]),
    WithDefault( String, PicoValue)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarOp {
    #[serde(rename = "var")]
    pub value: VarValue,

    #[serde(default)]
    pub register: Option<Vec<String>>,
    
    /**
     * is the var/value a JSONPath?
     */
    #[serde(default)]
    pub path: bool
}

impl Default for VarOp {
    fn default() -> Self {
        //Self { value: VarValue::String( "/".to_string()), register: None, path: false }
        Self { value:  VarValue::String("/".to_string()), path: false, register: None }
    }
}


/*
 * Statements
 */


#[derive(Serialize, Deserialize, Debug)]
pub struct LetStmt {

    #[serde(rename="let")]
    pub value: (String, Producer)
}

/**
 * declares a named variable with some JSON
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct SetStmt {

    #[serde(rename="set")]
    pub value: (String, PicoValue)
}




/*
 * Misc operations
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct DebugOperation {
    #[serde(rename = "debug")]
    value: String,
}
