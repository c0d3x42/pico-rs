pub mod der;

use jsonpath_lib::{compile, JsonPathError};
use serde_json::Value;

use std::convert::TryFrom;
use std::collections::HashMap;
use jmespatch;

use crate::PicoValue;


#[derive(thiserror::Error, Debug)]
pub enum PicoRuleError {
    #[error("Invalid PicoRule")]
    InvalidPicoRule,

    #[error("Invalid PicoVar")]
    InvalidPicoVar
}

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


impl TryFrom<der::RuleFile> for PicoRule {
   type Error = PicoRuleError;

    fn try_from(rule_file: der::RuleFile) -> Result<PicoRule, Self::Error> {
        let i = rule_file
            .root
            .into_iter()
            .map(|x| {
                let m = match x {
                    der::RuleInstruction::Logic(logic) => PicoInstruction::If(PicoIf::try_from(logic)?),
                    der::RuleInstruction::Let(l) => PicoInstruction::try_from(l)?,
                    der::RuleInstruction::Set(s) => PicoInstruction::try_from(s)?,
                    der::RuleInstruction::Debug(debug) => {
                        PicoInstruction::Debug(PicoInstructionDebug::try_from(debug)?)
                    }
                };
                Ok(m)
                //m
            })
            // https://stackoverflow.com/questions/62687455/alternatives-for-using-the-question-mark-operator-inside-a-map-function-closure
            .collect::<Result<Vec<PicoInstruction>, _>>()?;
        Ok(Self { root: i })
    }
}

impl PicoRule {

    pub fn run(&self) {
        let mut ctx: HashMap<String, PicoValue> = HashMap::new();
        for rule in &self.root {
            rule.run(&mut ctx);
        }
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

impl TryFrom<der::LetStmt> for PicoInstruction {
    type Error = PicoRuleError;

    fn try_from( s: der::LetStmt) -> Result<PicoInstruction, Self::Error> {
        let stmt = Self::Let{ varbind: s.value.0, value: Box::new(Expr::try_from(s.value.1)?)};
        Ok(stmt)
    }
}

impl TryFrom<der::SetStmt> for PicoInstruction {
    type Error = PicoRuleError;

    fn try_from( s: der::SetStmt) -> Result<PicoInstruction, Self::Error> {
        Ok( Self::Set{ varbind: s.value.0, value: s.value.1} )
    }
}

impl PicoInstruction {

    fn run(&self, ctx: &mut HashMap<String, PicoValue>) {
        //let mut ctx : HashMap<&String, &PicoValue>= HashMap::new();
        match &self {
            PicoInstruction::Set{ varbind , value} => {
                info!("SET varbind: {} ", varbind);
                ctx.insert(varbind.to_string(), value.clone());
            },
            PicoInstruction::Let{ varbind, value} => {

                let result = value.run(ctx);

                info!("LET varbind: {} = {:?}", varbind, result);
            }
            _ => {println!("unhandled command")}
        }
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

/**
 * ExprEq
 * 
 */
#[derive(Debug)]
pub struct ExprEq {
    lhs: Box<Expr>,
    rhs: Box<Expr>,
}
impl TryFrom<der::EqOperation> for ExprEq {
    type Error = PicoRuleError;

    fn try_from(eq_operation: der::EqOperation) -> Result<ExprEq, Self::Error> {
        Ok(Self {
            lhs: Box::new(Expr::try_from(eq_operation.value.0)?),
            rhs: Box::new(Expr::try_from(eq_operation.value.1)?),
        })
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

impl TryFrom<der::LessThanOperation> for ExprLt {
    type Error = PicoRuleError;

    fn try_from(lt_operation: der::LessThanOperation) -> Result<ExprLt, Self::Error> {
        let mut this = Self::default();
        if lt_operation.value.len() >= 2 {
            let mut iter = lt_operation.value.into_iter();

            if let Some(expr_first) = iter.next(){
                this.lhs = Box::new(Expr::try_from(expr_first)?);

                for expr in iter {
                    this.rhs.push( Expr::try_from(expr)?);
                }
            }
        }
        Ok(this)
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

impl TryFrom<der::Producer> for Expr {
    type Error = PicoRuleError;

    fn try_from(producer: der::Producer) -> Result<Expr, Self::Error> {
        println!("FRom: {:?}", producer);
        let prod = match producer {
            der::Producer::Eq(eq) => Expr::Eq(ExprEq::try_from(eq)?),
            der::Producer::Lt(lt) => Expr::Lt(ExprLt::try_from(lt)?),
            der::Producer::If(i) => Expr::If(Box::new(PicoIf::try_from(i)?)),
            der::Producer::Var(v) => Expr::Var(ExprVar::try_from(v)?),
            der::Producer::String(s) => Expr::String(s),
            _ => Expr::Nop,
        };

        Ok(prod)
    }
}

impl Expr{
    fn run(&self, ctx: &mut HashMap<String, PicoValue>) {
        match self {
            Expr::Var(v) => {
                for register in &v.registers {
                    debug!("Checking register {}", register);
                    if let Some(value) = ctx.get(register) {
                        debug!("Hash value {}", value);
                        if let Some(lookup) = value.pointer(&v.key){
                            info!("Found: {}", lookup);
                        } else {
                            debug!("Path missed {}", v.key);
                        }
                    } else {
                        info!("no register {}", register);
                    }
                }
                info!("VAR = {:?}", v);
            },
            _ => {}
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

impl TryFrom<der::IfOperation> for PicoIf {
    type Error = PicoRuleError;

    fn try_from(if_operation: der::IfOperation) -> Result<PicoIf, Self::Error> {

        let mut this = Self::default();

        let mut iter = if_operation.value.into_iter().peekable();

        while let Some(expr1) = iter.next() {
            if iter.peek().is_some() {
                if let Some(expr2) = iter.next() {
                    this.if_then.push((Expr::try_from(expr1)?, Expr::try_from(expr2)?));
                }
            } else {
                this.r#else = Some(Expr::try_from(expr1)?);
            }
        }

        Ok(this)
        //Err(PicoRuleError::InvalidPicoRule)
    }
}
#[derive(Debug)]
pub struct PicoInstructionDebug {
    instruction: String,
}
impl PicoInstructionTrait for PicoInstructionDebug {}

/*
impl From<der::DebugOperation> for PicoInstructionDebug {
    fn from(if_operation: der::DebugOperation) -> Self {
        Self {
            instruction: "".to_string(),
        }
    }
}
*/

impl TryFrom<der::DebugOperation> for PicoInstructionDebug {

    type Error = PicoRuleError;
    fn try_from(value: der::DebugOperation) -> Result<PicoInstructionDebug, Self::Error> {

        Ok(Self{ instruction: "ll".to_string()})
    }

}

#[derive(Debug)]
pub enum VarKeyType {
    Simple,
    JSONPointer,
    JSONPath,
    /**
     * https://lib.rs/crates/jmespatch
     */
    JMESPath,
    /**
     * https://crates.io/crates/json_dotpath
     */
    JSONDotPath,
    /**
     * https://crates.io/crates/jsondata
     */
    JSONData, 
}

#[derive(Debug)]
pub struct ExprVar {
    key: String,
    key_type: VarKeyType,
    default: PicoValue,

    registers: Vec<String>,
    jmespath: Option<jmespatch::Expression<'static>>


}

impl Default for ExprVar {
    fn default() -> Self {
        Self {
            key: "/".to_string(),
            key_type: VarKeyType::JSONPointer,
            default: PicoValue::Null,
            registers: Vec::new(),
            jmespath: None
        }
    }
}

impl ExprVar {

    fn exec(&self, value: &PicoValue){
        if let Some(jmespath) = &self.jmespath{
        }
    }
}

impl TryFrom<der::VarOp> for ExprVar {
    type Error = PicoRuleError;

    fn try_from(var: der::VarOp) -> Result<ExprVar, Self::Error> {
        let mut v: Self = Self {
            ..Default::default()
        };

        let j = jmespatch::compile("bar").unwrap();
        v.jmespath = Some(j);


        match var.value {
            der::VarValue::String(s) => v.key = s,
            der::VarValue::OneString(s) => {
                if let Some(s1) = s.first(){
                    v.key = s1.to_owned();
                }
            }

            der::VarValue::WithDefault(s,d) => {v.key = s; v.default = d;}

        }

        match var.register {
            der::VarRegister::Single(s) => v.registers.push(s),
            der::VarRegister::Named(registers) => v.registers = registers,
            _ => {}
        }
        
        Ok(v)
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