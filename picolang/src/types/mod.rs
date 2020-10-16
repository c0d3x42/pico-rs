pub mod der;
pub mod var;

use var::ExprVar;

use jsonpath_lib::{compile as jsonpath_compile, JsonPathError};
use serde_json::Value;

use jmespatch;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;
use log;

use crate::{PicoValue,pico_value_as_truthy};

#[derive(thiserror::Error, Debug)]
pub enum PicoRuleError {
    #[error("Invalid PicoRule")]
    InvalidPicoRule,

    #[error("Invalid PicoRule - unsupported expression {producer:?}")]
    UnsupportedExpression { producer: der::Producer },

    #[error("No such variable {variable:?}")]
    NoSuchVariable { variable: String},

    #[error("Invalid PicoVar")]
    InvalidPicoVar,
}

pub struct Context<'parent> {
    input: &'parent PicoValue,
    globals: &'parent HashMap<String, PicoValue>,
    locals: HashMap<String, PicoValue>,
    parent: Option<&'parent Self>,
}

// https://arzg.github.io/lang/6/
// Env
impl<'parent> Context<'parent> {
    pub fn new(input: &'parent PicoValue, globals: &'parent HashMap<String, PicoValue>) -> Self {
        Self {
            input,
            globals,
            locals: HashMap::new(),
            parent: None,
        }
    }

    pub fn create_child(&'parent self) -> Self {
        Self {
            input: &self.input,
            globals: &self.globals,
            locals: HashMap::new(),
            parent: Some(self),
        }
    }

    pub fn insert(&mut self, key: &str, value: &PicoValue) -> Option<PicoValue>{
        self.locals.insert(key.to_string(), value.clone())
    }

    pub fn get(&self, key: &str) -> Option<&PicoValue> {
        trace!("CTX get {}", key);
        self.locals.get(key)
    }

    pub fn input_get(&self, key: &str) -> Option<&PicoValue>{
        self.input.get(key)
    }
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
                    der::RuleInstruction::Logic(logic) => {
                        PicoInstruction::If(PicoIf::try_from(logic)?)
                    }
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

    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError>{
        let mut iter = self.root.iter().peekable();

        let result = loop {
            let i = iter.next();
            match i {
                Some(instruction) => {
                    let last = instruction.run(ctx);
                    if iter.peek().is_none() {
                        break last;
                    }

                },
                None => break Ok(PicoValue::Null)
            }
        };

        result
    }

    pub fn run(&self, ctx: &mut Context) {
        for rule in &self.root {
            match rule.run(ctx) {
                Ok(result) => { info!("PicoRule result {}", result)},
                Err(pre) => {error!("PicoRule {}", pre)}
            }
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
    Let { varbind: String, value: Box<Expr> },
    Set { varbind: String, value: PicoValue },
    Debug(PicoInstructionDebug),
}

impl TryFrom<der::LetStmt> for PicoInstruction {
    type Error = PicoRuleError;

    fn try_from(s: der::LetStmt) -> Result<PicoInstruction, Self::Error> {
        let stmt = Self::Let {
            varbind: s.value.0,
            value: Box::new(Expr::try_from(s.value.1)?),
        };
        Ok(stmt)
    }
}

impl TryFrom<der::SetStmt> for PicoInstruction {
    type Error = PicoRuleError;

    fn try_from(s: der::SetStmt) -> Result<PicoInstruction, Self::Error> {
        Ok(Self::Set {
            varbind: s.value.0,
            value: s.value.1,
        })
    }
}

impl PicoInstruction {
    fn run(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError>{
        //let mut ctx : HashMap<&String, &PicoValue>= HashMap::new();
        match &self {
            PicoInstruction::Set { varbind, value } => {
                info!("SET varbind: {} ", varbind);
                ctx.insert(varbind, value).ok_or(PicoRuleError::InvalidPicoRule)
            }
            PicoInstruction::Let { varbind, value } => {
                let result = value.run(ctx);

                info!("LET varbind: {} = {:?}", varbind, result);

                result
            },
            PicoInstruction::If(pif) => pif.exec(ctx),
            _ => Err(PicoRuleError::InvalidPicoRule)
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

impl ExprEq{

    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError>{
        trace!("ExprEq...");
        let left_handle_value = self.lhs.run(ctx)?;
        let right_handle_value = self.rhs.run(ctx)?;
        trace!("ExprEq {} == {}", left_handle_value, right_handle_value);

        Ok( PicoValue::Bool( left_handle_value == right_handle_value ))
    }
}

#[derive(Debug)]
pub struct ExprLt {
    lhs: Box<Expr>,
    rhs: Vec<Expr>,
}

impl Default for ExprLt {
    fn default() -> Self {
        Self {
            lhs: Box::new(Expr::Nop),
            rhs: Vec::new(),
        }
    }
}

impl TryFrom<der::LessThanOperation> for ExprLt {
    type Error = PicoRuleError;

    fn try_from(lt_operation: der::LessThanOperation) -> Result<ExprLt, Self::Error> {
        let mut this = Self::default();
        if lt_operation.value.len() < 2 {
            return Err(PicoRuleError::InvalidPicoRule);
        }

        let mut iter = lt_operation.value.into_iter();

        if let Some(expr_first) = iter.next() {
            this.lhs = Box::new(Expr::try_from(expr_first)?);

            for expr in iter {
                this.rhs.push(Expr::try_from(expr)?);
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
            _ => return Err(PicoRuleError::UnsupportedExpression { producer }),
        };

        Ok(prod)
    }
}

impl Expr {
    fn run(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError>{

        trace!("Expr {:?}", self);
        match self {
            Expr::Var(v) => {
                info!("VAR = {:?}", v);
                v.exec( ctx)
            },
            Expr::Eq(eq)=> eq.exec(ctx),
            Expr::String(s) => Ok(PicoValue::String(s.to_string())),

            _ => Err(PicoRuleError::InvalidPicoRule)
        }
    }
}

#[derive(Debug)]
pub struct PicoIf {
    r#if_then: Vec<(Expr, Expr)>,
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

        if if_operation.value.is_empty() {
            return Err(PicoRuleError::InvalidPicoRule);
        }

        let mut iter = if_operation.value.into_iter().peekable();

        while let Some(expr1) = iter.next() {
            if iter.peek().is_some() {
                if let Some(expr2) = iter.next() {
                    this.if_then
                        .push((Expr::try_from(expr1)?, Expr::try_from(expr2)?));
                }
            } else {
                this.r#else = Some(Expr::try_from(expr1)?);
            }
        }

        Ok(this)
        //Err(PicoRuleError::InvalidPicoRule)
    }
}

impl PicoIf {
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError>{
        trace!("PicoIf");

        for (if_stmt, then_stmt) in &self.if_then {
            let res = if_stmt.run(ctx)?;
            if pico_value_as_truthy(&res){
                trace!("PicoIf..Then");
                return then_stmt.run(ctx);
            }
        }

        if let Some(else_stmt) = &self.r#else {
            trace!("PicoIf..Else {:?}", else_stmt);
            return else_stmt.run(ctx);
        }

        trace!("PicoIf not matched");

        Ok(PicoValue::Null)

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
        Ok(Self {
            instruction: "ll".to_string(),
        })
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
