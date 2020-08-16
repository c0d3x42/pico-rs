use serde::{Deserialize, Serialize};
use std::collections::HashMap;
//use std::rc::Rc;

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::context::PicoContext;
use crate::errors::PicoError;
use crate::state::PicoState;
use crate::PicoValue;

pub type LookupDict = HashMap<String, PicoValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupTable {
    pub entries: LookupDict,
    pub default: PicoValue,
}

impl Default for LookupTable {
    fn default() -> Self {
        Self {
            default: PicoValue::String("unknown".to_string()),
            entries: HashMap::new(),
        }
    }
}

impl LookupTable {
    pub fn new() -> LookupTable {
        Default::default()
    }

    pub fn lookup(&self, key: &str) -> &PicoValue {
        if let Some(value) = self.entries.get(key) {
            value
        } else {
            &self.default
        }
    }
}

//pub type Lookups = HashMap<String, Rc<LookupTable>>;
pub type Lookups = HashMap<String, LookupTable>;

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupCommand {
    lookup: (String, String), // table, key
}

impl Execution for LookupCommand {
    fn name(&self) -> String {
        String::from("lookup")
    }

    fn run_with_context(&self, state: &mut PicoState, _ctx: &mut PicoContext) -> FnResult {
        info!(
            "Lookup Dictionary {:?} -> {:?}",
            self.lookup.0, self.lookup.1
        );

        if let Some(v) = state.get_lookup_value(&self.lookup.0, &self.lookup.1) {
            match v {
                PicoValue::String(s) => {
                    return Ok(ExecutionResult::Continue(PicoValue::String(s.to_string())))
                }
                _ => {
                    return Err(PicoError::NoSuchValue(format!(
                        "{}/{}",
                        &self.lookup.0, &self.lookup.1
                    )))
                }
            }
        }

        info!("Lookup failed for {:?}", self.lookup.0);

        Err(PicoError::NoSuchValue(format!(
            "No Such table {}",
            &self.lookup.0
        )))
    }
}
