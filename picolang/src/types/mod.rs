pub mod and;
pub mod block;
pub mod debug;
pub mod der;
pub mod eq;
pub mod r#let;
pub mod logic;
pub mod lt;
pub mod or;
pub mod run;
pub mod set;
pub mod var;

use and::ExprAnd;
use block::{ExprBlock, ExprStop};
use debug::ExprDebug;
use eq::ExprEq;
use lt::ExprLt;
use or::ExprOr;
use r#let::ExprLet;
use set::ExprSet;
use var::ExprVar;

use jsonpath_lib::{compile as jsonpath_compile, JsonPathError};
use serde_json::Value;

use jmespatch;
use log;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

use crate::{pico_value_as_truthy, PicoValue};

#[derive(thiserror::Error, Debug)]
pub enum PicoRuleError {
    #[error("Invalid PicoRule")]
    InvalidPicoRule,

    #[error("Invalid PicoRule - unsupported expression {producer:?}")]
    UnsupportedExpression { producer: der::Producer },

    #[error("No such variable {variable:?}")]
    NoSuchVariable { variable: String },

    #[error("Invalid PicoVar")]
    InvalidPicoVar,

    #[error("Stopping a block")]
    BlockStop,
}

pub struct Message {
    msg: String,
}
impl Message {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

pub struct Context<'parent> {
    input: &'parent PicoValue,
    globals: &'parent HashMap<String, PicoValue>,
    locals: HashMap<String, PicoValue>,
    parent: Option<&'parent Self>,
    messages: Vec<Message>,
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
            messages: Vec::new(),
        }
    }

    pub fn create_child(&'parent self) -> Self {
        Self {
            input: &self.input,
            globals: &self.globals,
            locals: HashMap::new(),
            parent: Some(self),
            messages: Vec::new(),
        }
    }

    pub fn add_msg(&mut self, msg: &str) {
        self.messages.push(Message::new(msg));
    }

    pub fn insert(&mut self, key: &str, value: &PicoValue) -> Option<PicoValue> {
        self.locals.insert(key.to_string(), value.clone())
    }

    pub fn get(&self, key: &str) -> Option<&PicoValue> {
        trace!("CTX get {}", key);

        if self.locals.contains_key(key) {
            self.locals.get(key)
        } else if let Some(p) = self.parent {
            p.get(key)
        } else {
            None
        }
    }

    pub fn input_get(&self, key: &str) -> Option<&PicoValue> {
        self.input.get(key)
    }
}

#[derive(Debug)]
pub struct PicoRule {
    root: Vec<Expr>,
}

impl TryFrom<der::RuleFile> for PicoRule {
    type Error = PicoRuleError;

    fn try_from(rule_file: der::RuleFile) -> Result<PicoRule, Self::Error> {
        let mut instructions: Vec<Expr> = Vec::new();

        match rule_file.root {
            der::JsonLogic::Single(producer) => {
                instructions.push(Expr::try_from(producer)?);
            }
            der::JsonLogic::Many(many_producers) => {
                for producer in many_producers {
                    instructions.push(Expr::try_from(producer)?);
                }
            }
            _ => {}
        }

        Ok(Self { root: instructions })
    }
}

impl PicoRule {
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
        trace!("PicoRule::exec");
        let mut results: Vec<PicoValue> = Vec::new();

        for expr in &self.root {
            trace!("running expr");
            // FIXME this swallows Stop
            //let result: PicoValue = expr.run(ctx).unwrap_or(PicoValue::Null);
            let result: PicoValue = expr.run(ctx)?;
            results.push(result);
        }

        trace!("PicoRule result count {}", results.len());

        match results.len() {
            0 => Ok(PicoValue::Null),
            1 => Ok(results.pop().unwrap_or(PicoValue::Null)),
            _ => Ok(PicoValue::Array(results)),
        }
    }

    pub fn run(&self, ctx: &mut Context) -> Result<PicoValue, String> {
        let result = self.exec(ctx);

        for msg in &ctx.messages {
            info!("MSG {}", msg.msg);
        }
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                error!("PicoRule::run {}", err);
                Err(format!("{}", err))
            }
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

#[derive(Debug)]
pub enum Expr {
    Nop,
    // statementy
    If(Box<PicoIf>),
    Let(ExprLet),
    Set(ExprSet),
    Debug(ExprDebug),
    // expressiony
    Block(ExprBlock),
    Stop(ExprStop),
    Eq(ExprEq),
    Lt(ExprLt),
    And(ExprAnd),
    Or(ExprOr),
    Var(ExprVar),
    String(String),
}

impl TryFrom<der::Producer> for Expr {
    type Error = PicoRuleError;

    fn try_from(producer: der::Producer) -> Result<Expr, Self::Error> {
        println!("TryFrom:Expr {:?}", producer);
        let prod = match producer {
            der::Producer::Eq(eq) => Expr::Eq(ExprEq::try_from(eq)?),
            der::Producer::Lt(lt) => Expr::Lt(ExprLt::try_from(lt)?),
            der::Producer::If(i) => Expr::If(Box::new(PicoIf::try_from(i)?)),
            der::Producer::And(a) => Expr::And(ExprAnd::try_from(a)?),
            der::Producer::Or(o) => Expr::Or(ExprOr::try_from(o)?),
            der::Producer::Let(l) => Expr::Let(ExprLet::try_from(l)?),
            der::Producer::Set(s) => Expr::Set(ExprSet::try_from(s)?),
            der::Producer::Var(v) => Expr::Var(ExprVar::try_from(v)?),
            der::Producer::Block(b) => Expr::Block(ExprBlock::try_from(b)?),
            der::Producer::Stop(s) => Expr::Stop(ExprStop::try_from(s)?),

            der::Producer::Debug(d) => Expr::Debug(ExprDebug::try_from(d)?),
            der::Producer::String(s) => Expr::String(s),
            _ => return Err(PicoRuleError::UnsupportedExpression { producer }),
        };

        Ok(prod)
    }
}

impl Expr {
    fn run(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
        trace!("Expr {:?}", self);

        match self {
            Expr::Let(l) => l.exec(ctx),
            Expr::Set(s) => s.exec(ctx),
            Expr::Debug(d) => d.exec(ctx),

            Expr::Var(v) => v.exec(ctx),
            Expr::Eq(eq) => eq.exec(ctx),
            Expr::And(a) => a.exec(ctx),

            Expr::If(i) => i.exec(ctx),
            Expr::Lt(l) => l.exec(ctx),

            Expr::Block(b) => b.exec(ctx),
            Expr::Stop(s) => s.exec(ctx),

            Expr::String(s) => Ok(PicoValue::String(s.to_string())),

            _ => Err(PicoRuleError::InvalidPicoRule),
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

impl TryFrom<der::IfStmt> for PicoIf {
    type Error = PicoRuleError;

    fn try_from(if_operation: der::IfStmt) -> Result<PicoIf, Self::Error> {
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
    pub fn exec(&self, ctx: &mut Context) -> Result<PicoValue, PicoRuleError> {
        trace!("PicoIf");

        for (if_stmt, then_stmt) in &self.if_then {
            let res = if_stmt.run(ctx)?;
            if pico_value_as_truthy(&res) {
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
