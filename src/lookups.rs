use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::command::{Execution, ExecutionResult, FnResult};
use crate::context::{Context, PicoState};
use crate::errors::PicoError;
use crate::PicoValue;

pub type LookupDict = HashMap<String, PicoValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupTable {
    pub entries: LookupDict,
    pub default: PicoValue,
}

impl LookupTable {
    pub fn new() -> LookupTable {
        LookupTable {
            default: PicoValue::String("unknown".to_string()),
            entries: HashMap::new(),
        }
    }

    pub fn lookup(&self, key: &String) -> &PicoValue {
        if let Some(value) = self.entries.get(key) {
            return value;
        } else {
            return &self.default;
        }
    }
}

pub type Lookups = HashMap<String, LookupTable>;

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupCommand {
    lookup: (String, String), // table, key
}

impl Execution for LookupCommand {
    fn name(&self) -> String {
        String::from("lookup")
    }

    fn run_with_context(&self, state: &PicoState, _ctx: &mut Context) -> FnResult {
        info!(
            "Lookup Dictionary {:?} -> {:?}",
            self.lookup.0, self.lookup.1
        );

        if let Some(t) = state.lookup_tables.get(&self.lookup.0) {
            if let Some(value) = t.entries.get(&self.lookup.1) {
                match value {
                    PicoValue::String(s) => {
                        trace!("Found lookup value {:?}", s);
                        return Ok(ExecutionResult::Continue(PicoValue::String(s.to_string())));
                    }
                    _ => return Err(PicoError::NoSuchValue),
                }
            } else {
                trace!("lookup using default {:?}", t.default);
                match &t.default {
                    PicoValue::String(s) => {
                        return Ok(ExecutionResult::Continue(PicoValue::String(s.to_string())))
                    }
                    _ => return Err(PicoError::NoSuchValue),
                }
            }
        }

        info!("Lookup failed for {:?}", self.lookup.0);

        Err(PicoError::NoSuchValue)
    }
}
