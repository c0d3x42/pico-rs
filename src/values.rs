use serde::{Deserialize, Serialize};

use crate::command::{Execution, ExecutionResult, FnResult};
use crate::context::{Context, VariablesMap};
use crate::errors::PicoError;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PicoValue {
    UnsignedNumber(usize),
    Number(isize),
    String(String),
    Boolean(bool),
}

impl PartialEq for PicoValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&PicoValue::Boolean(a), &PicoValue::Boolean(b)) => a == b,
            (&PicoValue::UnsignedNumber(a), &PicoValue::UnsignedNumber(b)) => a == b,
            (&PicoValue::Number(a), &PicoValue::Number(b)) => a == b,
            (&PicoValue::String(ref a), &PicoValue::String(ref b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for PicoValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (&PicoValue::UnsignedNumber(a), &PicoValue::UnsignedNumber(b)) => Some(a.cmp(&b)),
            (&PicoValue::Number(a), &PicoValue::Number(b)) => Some(a.cmp(&b)),
            (&PicoValue::String(ref a), &PicoValue::String(ref b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

#[test]
fn eq9() {
    let v = PicoValue::Number(9);
    assert_eq!(v.eq(&PicoValue::Number(9)), true);
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum VarValue {
    Lookup(String),
    DefaultLookup(String, PicoValue),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarLookup {
    var: VarValue,
}

impl Execution for VarLookup {
    fn name(&self) -> String {
        return "VarLookup".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        match &self.var {
            // Plain lookup in ctx variables
            VarValue::Lookup(s) => {
                debug!("lookup {:?}", s);
                let lookup = ctx.variables.get(s);
                match lookup {
                    Some(v) => {
                        let r = v.clone();
                        return Ok(ExecutionResult::Continue(r));
                    }
                    None => {
                        info!("Failed to lookup var {:?}", s);
                        let local_lookup = ctx.local_variables.get(s);
                        match local_lookup {
                            Some(v) => return Ok(ExecutionResult::Continue(v.clone())),
                            None => return Err(PicoError::NoSuchValue),
                        }
                    }
                };
            }
            VarValue::DefaultLookup(varname, fallback) => {
                debug!("default lookup {:?}, {:?}", varname, fallback);

                let lookup = ctx.variables.get(varname);
                match lookup {
                    Some(value) => return Ok(ExecutionResult::Continue(value.clone())),
                    None => return Ok(ExecutionResult::Continue(fallback.clone())),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Var {
    Literal(PicoValue),
    Lookup(VarLookup),
}

impl Execution for Var {
    fn name(&self) -> String {
        return "Var".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        match self {
            Var::Lookup(lookup) => lookup.run_with_context(ctx),
            Var::Literal(literal) => Ok(ExecutionResult::Continue(literal.clone())),
        }
    }
}
