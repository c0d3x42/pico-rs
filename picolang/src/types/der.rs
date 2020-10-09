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
 * Variables
 */

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum VarValue {
    Simple(String),
    WithDefault(String, String),
}

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
pub struct VarOp {
    #[serde(rename = "var")]
    pub value: VarValue,
    #[serde(default = "VarType::plain")]
    pub r#type: VarType,

    #[serde(default = "VarOp::register")]
    pub register: String,
}

impl VarOp {
    fn register() -> String {
        "_".to_string()
    }
}

/*
 * Values
 */

/*
 * Misc operations
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct DebugOperation {
    #[serde(rename = "debug")]
    value: String,
}
