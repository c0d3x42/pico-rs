use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::command::{Execution, FnResult};
use crate::context::Context;
use crate::errors::PicoError;
use crate::{PicoValue, ValueProducer};

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
    lookup: String,
}

impl Execution for LookupCommand {
    fn name(&self) -> String {
        String::from("lookup")
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        info!("Lookup Dictionary not impl");
        Err(PicoError::Crash(String::from("not done")))
    }
}
