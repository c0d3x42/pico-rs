pub mod der;

use jsonpath_lib::{compile, JsonPathError};
use serde_json::Value;

use std::convert::TryFrom;

use crate::PicoValue;

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
                der::RuleInstruction::Let(l) => PicoInstruction::Let(PicoLet::from(l)),
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
    Let(PicoLet),
    Debug(PicoInstructionDebug),
}

#[derive(Debug)]
pub struct PicoLet {
    varbind: String,
    value: Box<Expr>
}

impl From<der::LetStmt> for PicoLet {
    fn from( source: der::LetStmt) -> Self{
        Self { varbind: source.value.0, value: Box::new(Expr::from(source.value.1))}
    }
}


#[derive(Debug)]
pub struct ExprString {
    s: String,
}
impl From<String> for ExprString {
    fn from(s: String) -> Self {
        Self { s: s.clone() }
    }
}

#[derive(Debug)]
pub struct ExprEq {
    lhs: Box<Expr>,
    rhs: Box<Expr>,
}
impl From<der::EqOperation> for ExprEq {
    fn from(eq_operation: der::EqOperation) -> Self {
        Self {
            lhs: Box::new(Expr::from(eq_operation.value.0)),
            rhs: Box::new(Expr::from(eq_operation.value.1)),
        }
    }
}

#[derive(Debug)]
pub struct ExprLt {
    lhs: Box<Expr>,
    rhs: Vec<Expr>,
}

impl Default for ExprLt {
    fn default() ->Self{
        Self{ lhs: Box::new(Expr::Nop), rhs: Vec::new()}
    }
}

impl From<der::LessThanOperation> for ExprLt {
    fn from(lt_operation: der::LessThanOperation) -> Self {
        let mut this = Self::default();
        if lt_operation.value.len() >= 2 {
            let mut iter = lt_operation.value.into_iter();

            if let Some(expr_first) = iter.next(){
                this.lhs = Box::new(Expr::from(expr_first));

                for expr in iter {
                    this.rhs.push( Expr::from(expr));
                }
            }
        }
        this
    }
}



#[derive(Debug)]
pub enum Expr {
    Nop,
    Eq(ExprEq),
    Lt(ExprLt),
    If(Box<PicoIf>),
    String(String),
}

impl From<der::Producer> for Expr {
    fn from(producer: der::Producer) -> Self {
        println!("FRom: {:?}", producer);
        match producer {
            der::Producer::Eq(eq) => Expr::Eq(ExprEq::from(eq)),
            der::Producer::Lt(lt) => Expr::Lt(ExprLt::from(lt)),
            der::Producer::If(i) => Expr::If(Box::new(PicoIf::from(i))),
            der::Producer::String(s) => Expr::String(s),
            _ => Expr::Nop,
        }
    }
}

#[derive(Debug)]
pub struct PicoIf {
    r#if: Expr,
    r#then: Expr,
    elseifs: Vec<(Expr, Expr)>,
    r#else: Option<Expr>,
}

impl Default for PicoIf {
    fn default() -> Self {
        Self {
            r#if: Expr::Nop,
            r#then: Expr::Nop,
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

        let mut iter = if_operation.value.into_iter().peekable();
        if let Some(first_if) = iter.next() {
            if let Some(first_then) = iter.next() {
                this.r#if = Expr::from(first_if);
                this.r#then = Expr::from(first_then);
            }
        }

        while let Some(expr1) = iter.next() {
            if let Some(expr2_peek) = iter.peek() {
                if let Some(expr2) = iter.next() {
                    this.elseifs.push((Expr::from(expr1), Expr::from(expr2)));
                }
            } else {
                this.r#else = Some(Expr::from(expr1));
            }
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
