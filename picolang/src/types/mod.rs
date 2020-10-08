pub mod der;

use jsonpath_lib::{compile, JsonPathError};
use serde_json::Value;

use std::convert::TryFrom;

#[derive(Debug)]
pub struct PicoRule {
    root: Vec<PicoInstruction>,
}
/*
impl std::fmt::Debug for PicoRule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.root.len())
    }
}
*/

impl From<der::RuleFile> for PicoRule {
    fn from(rule_file: der::RuleFile) -> Self {
        let i = rule_file
            .root
            .into_iter()
            .map(|x| match x {
                der::RuleInstruction::Logic(logic) => PicoInstruction::If(PicoIf::from(logic)),
                der::RuleInstruction::Debug(debug) => {
                    PicoInstruction::Debug(PicoInstructionDebug::from(debug))
                }
            })
            .collect();
        Self { root: i }
    }
}
/*
impl From<der::RuleFile> for PicoRule {
    fn from(rule_file: der::RuleFile) -> Self {
        let instructions: Vec<Box<PicoInstruction>> = rule_file
            .root
            .into_iter()
            .map(|i| match i {
                der::RuleInstruction::Logic(_) => PicoInstruction::from,
            })
            .collect();
        Self { root: instructions }
    }
}
*/

pub trait PicoInstructionTrait {
    fn exec(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub enum PicoInstruction {
    If(PicoIf),
    Debug(PicoInstructionDebug),
}

#[derive(Debug)]
pub enum Producer {
    One,
    Nop,
}

#[derive(Debug)]
pub struct PicoIf {
    r#if: Producer,
    r#then: Producer,
    elseifs: Vec<(Producer, Producer)>,
    r#else: Option<Producer>,
}

impl Default for PicoIf {
    fn default() -> Self {
        Self {
            r#if: Producer::Nop,
            r#then: Producer::Nop,
            elseifs: Vec::new(),
            r#else: None,
        }
    }
}

impl PicoInstructionTrait for PicoIf {}
impl From<der::IfOperation> for PicoIf {
    fn from(if_operation: der::IfOperation) -> Self {
        let counter = if_operation.value.len();

        let mut this = Self::default();

        for p in if_operation.value.chunks(2) {
            println!("L={}, P = {:?}", if_operation.value.len(), p);
        }

        for x in &if_operation.value {}
        let mut iter = if_operation.value.into_iter();

        while let Some(p) = iter.next() {}

        match counter {
            i if i <= 2 => {
                this.r#if = match iter.next() {
                    Some(x) => Producer::One,
                    None => Producer::Nop,
                }
            }
            _ => {}
        }

        this
    }
}
#[derive(Debug)]
pub struct PicoInstructionDebug {
    instruction: String,
}
impl PicoInstructionTrait for PicoInstructionDebug {}
impl From<der::DebugOperation> for PicoInstructionDebug {
    fn from(if_operation: der::DebugOperation) -> Self {
        Self {
            instruction: "".to_string(),
        }
    }
}

struct PicoVarOp {
    key: String,
    default: Option<String>,

    vartype: der::VarType,

    register: String,
}

impl Default for PicoVarOp {
    fn default() -> Self {
        Self {
            key: "".to_string(),
            default: None,
            vartype: der::VarType::plain(),
            register: "_".to_string(),
        }
    }
}

impl From<der::VarOp> for PicoVarOp {
    fn from(var: der::VarOp) -> Self {
        let mut v: Self = Self {
            register: String::from(&var.register),
            vartype: var.r#type,
            ..Default::default()
        };

        match &var.value {
            der::VarValue::Simple(s) => v.key = String::from(s),
            der::VarValue::WithDefault(s, def) => {
                v.key = String::from(s);
                v.default = Some(String::from(def))
            }
        }

        v
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn make_pico_var() {
        let varOp: der::VarOp = der::VarOp {
            value: der::VarValue::Simple("lop".to_string()),
            register: "_".to_string(),
            r#type: der::VarType::Path,
        };

        let picoVar = PicoVarOp::from(varOp);
    }
}
