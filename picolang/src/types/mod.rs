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
                der::RuleInstruction::Let(l) => PicoInstruction::from(l),
                der::RuleInstruction::Set(s) => PicoInstruction::from(s),
                der::RuleInstruction::Debug(debug) => {
                    PicoInstruction::Debug(PicoInstructionDebug::from(debug))
                }
            })
            .collect();
        Self { root: i }
    }
}

pub trait PicoInstructionTrait {
    fn exec(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub enum PicoInstruction {
    If(PicoIf),
    Let { varbind: String, value: Box<Expr>},
    Set {varbind: String, value: PicoValue},
    Debug(PicoInstructionDebug),
}

impl From<der::LetStmt> for PicoInstruction {
    fn from( s: der::LetStmt) -> Self {
        Self::Let{ varbind: s.value.0, value: Box::new(Expr::from(s.value.1))}
    }
}

impl From<der::SetStmt> for PicoInstruction {
    fn from( s: der::SetStmt) -> Self {
        Self::Set{ varbind: s.value.0, value: s.value.1}
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
    Var(ExprVar),
    String(String),
}

impl From<der::Producer> for Expr {
    fn from(producer: der::Producer) -> Self {
        println!("FRom: {:?}", producer);
        match producer {
            der::Producer::Eq(eq) => Expr::Eq(ExprEq::from(eq)),
            der::Producer::Lt(lt) => Expr::Lt(ExprLt::from(lt)),
            der::Producer::If(i) => Expr::If(Box::new(PicoIf::from(i))),
            der::Producer::Var(v) => Expr::Var(ExprVar::from(v)),
            der::Producer::String(s) => Expr::String(s),
            _ => Expr::Nop,
        }
    }
}

#[derive(Debug)]
pub struct PicoIf {
    r#if_then: Vec<(Expr,Expr)>,
    r#else: Option<Expr>,
}

impl Default for PicoIf {
    fn default() -> Self {
        Self {
            if_then: Vec::new(),
            r#else: None,
        }
    }
}

impl PicoInstructionTrait for PicoIf {}
impl From<der::IfOperation> for PicoIf {
    fn from(if_operation: der::IfOperation) -> Self {

        let mut this = Self::default();

        let mut iter = if_operation.value.into_iter().peekable();

        while let Some(expr1) = iter.next() {
            if iter.peek().is_some() {
                if let Some(expr2) = iter.next() {
                    this.if_then.push((Expr::from(expr1), Expr::from(expr2)));
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

#[derive(Debug)]
pub struct ExprVar {
    key: String,
    default: PicoValue,

    registers: Vec<String>,

    json_path: bool

}

impl Default for ExprVar {
    fn default() -> Self {
        Self {
            key: "/".to_string(),
            default: PicoValue::Null,
            registers: Vec::new(),
            json_path: false
        }
    }
}

impl From<der::VarOp> for ExprVar {
    fn from(var: der::VarOp) -> Self {
        let mut v: Self = Self {
            ..Default::default()
        };

        match var.value {
            der::VarValue::String(s) => v.key = s,
            der::VarValue::OneString(s) => {
                if let Some(s1) = s.first(){
                    v.key = s1.to_owned();
                }
            }

            der::VarValue::WithDefault(s,d) => {v.key = s; v.default = d;}

        }
        

        
        if let Some(registers) = var.register {
            v.registers = registers;
        }
        v.json_path = var.path;

        v
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn make_pico_var() {
        let varOp: der::VarOp = der::VarOp {
            value: ("l".to_string(), None),
            register: None,
            path: false
        };

        let picoVar = PicoVarOp::from(varOp);
    }
}
*/