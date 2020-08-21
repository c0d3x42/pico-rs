use serde::de::Visitor;
use serde::{Deserialize, Serialize};

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::context::PicoContext;
//use crate::state::PicoState;
use crate::loader::PicoRules;
use crate::loader::PicoRuntime as PicoState;
//use crate::values::{PicoValue, Var};
use crate::PicoValue;

use regex::Regex;

/*
 * existance or not of variables in the context
 */

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum VarExistence {
    SingleVar(String),
    ManyVar(Vec<String>),
}

#[derive(Debug)]
struct VarExistsVisitor;
impl<'de> Visitor<'de> for VarExistsVisitor {
    type Value = VarExistence;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        error!("varexistvistior: expecting {:?}", self);
        formatter.write_str("string or array of strings")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        info!("Checking value: {}", value);
        let re: Regex = Regex::new("^[a-z]+$").unwrap();
        if re.is_match(value) {
            return Ok(VarExistence::SingleVar(value.to_string()));
        }

        //error!("exists must be lower case");
        //Err(E::custom("llll"))
        Ok(VarExistence::SingleVar(value.to_string()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::StdError,
    {
        Ok(VarExistence::SingleVar(value.to_string()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut arr = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(item) = seq.next_element()? {
            arr.push(item);
        }

        info!("VarExistVisitor - seq {:?}", arr);
        Ok(VarExistence::ManyVar(arr))
    }
}

impl<'de> Deserialize<'de> for VarExistence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(VarExistsVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarExistsCondition {
    exists: VarExistence,
}

impl Execution for VarExistsCondition {
    fn name(&self) -> String {
        "VarExists".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        _state: &mut PicoState,
        ctx: &mut PicoContext,
    ) -> FnResult {
        match &self.exists {
            VarExistence::SingleVar(s) => {
                if let Some(_v) = ctx.get_value(s) {
                    return Ok(ExecutionResult::Continue(PicoValue::Boolean(true)));
                }
            }
            VarExistence::ManyVar(vs) => {
                for v in vs {
                    match ctx.get_value(v) {
                        None => return Ok(ExecutionResult::Continue(PicoValue::Boolean(false))),
                        Some(_) => {}
                    }
                }
                // not yet exited early, all vars existed
                return Ok(ExecutionResult::Continue(PicoValue::Boolean(true)));
            }
        }

        Ok(ExecutionResult::Continue(PicoValue::Boolean(false)))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarMissingCondition {
    missing: String,
}
impl Execution for VarMissingCondition {
    fn name(&self) -> String {
        "VarMissing".to_string()
    }

    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _state: &mut PicoState,
        ctx: &mut PicoContext,
    ) -> FnResult {
        let final_result = match ctx.get_value(&self.missing) {
            Some(_v) => Ok(ExecutionResult::Continue(PicoValue::Boolean(false))),
            None => Ok(ExecutionResult::Continue(PicoValue::Boolean(true))),
        };
        final_result
    }
}
