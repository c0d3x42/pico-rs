use crate::PicoValue;
use serde::Deserialize;
use std::collections::HashMap;

pub type LookupTableName = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum JsonLogic {
    Single(Producer),
    Many(Vec<Producer>)
}

///
/// The ondisk representation of a Pico rule file
#[derive(Serialize, Deserialize, Debug)]
pub struct RuleFile {
    #[serde(default)]
    pub version: String,

    #[serde(default)]
    pub lookups: HashMap<LookupTableName, LookupDefinition>,

    pub root: JsonLogic
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
    /**
     * statementy
     */
    If(IfStmt),
    Let(LetStmt),
    Set(SetStmt),
    Debug(DebugStmt),

    /**
     * expressiony
     */
    Eq(EqOperation),
    Ne(NeOperation),
    Or(OrOperation),
    Lt(LessThanOperation),
    And(AndOperation),
    Var(VarOp),
    String(String),
}

/*
 * Logic operations
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct IfStmt {
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
    pub value: Box<(Producer, Producer)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrOperation {
    #[serde(rename = "or")]
    pub value: Vec<Producer>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AndOperation {
    #[serde(rename = "and")]
    pub value: Vec<Producer>,
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
 * Array Operations
 */



/*
 * Variable lookup
 */


#[derive(Serialize, Deserialize, Debug)]
pub enum VarType {
    #[serde(alias="Type not detected")]
    Unknown,
    Plain,
    #[serde(rename="pointer")]
    Pointer,
    #[serde(rename="path")]
    Path
}
impl VarType {
    pub fn plain() -> Self {
        VarType::Plain
    }
}
impl Default for VarType {
    fn default() -> Self {
        Self::Plain
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
#[serde(untagged)]
pub enum VarRegister {
    Input,
    Single(String),
    Named(Vec<String>)
}
impl Default for VarRegister{
    fn default() -> Self {
        Self::Input
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarOp {
    #[serde(rename = "var")]
    pub value: VarValue,

    #[serde(default)]
    pub register: VarRegister,
    
    /**
     * is the var/value a JSONPath?
     */
    #[serde(default )]
    pub r#type: VarType

}

impl Default for VarOp {
    fn default() -> Self {
        //Self { value: VarValue::String( "/".to_string()), register: None, path: false }
        Self { value:  VarValue::String("/".to_string()), r#type: VarType::Plain, register: VarRegister::Input }
    }
}


/*
 * Statements
 */


#[derive(Serialize, Deserialize, Debug)]
pub struct LetStmt {

    #[serde(rename="let")]
    pub value: (String, Box<Producer>)
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
pub struct DebugStmt {
    #[serde(rename = "debug")]
    pub value: String,
}
