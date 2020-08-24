use serde::de::Visitor;
use serde::{Deserialize, Serialize};

use crate::commands::execution::{ConditionExecution, ConditionResult};
use crate::context::PicoContext;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
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

impl ConditionExecution for VarExistsCondition {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        _runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        match &self.exists {
            VarExistence::SingleVar(s) => {
                if let Some(_v) = ctx.get_value(s) {
                    return Ok(true);
                }
            }
            VarExistence::ManyVar(vs) => {
                for v in vs {
                    match ctx.get_value(v) {
                        None => return Ok(false),
                        Some(_) => {}
                    }
                }
                // not yet exited early, all vars existed
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarMissingCondition {
    missing: String,
}
impl ConditionExecution for VarMissingCondition {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ConditionResult {
        match ctx.get_value(&self.missing) {
            Some(_v) => Ok(false),
            None => Ok(true),
        }
    }
}
