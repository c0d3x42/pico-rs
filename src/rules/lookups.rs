use serde::{Deserialize, Serialize};
use std::collections::HashMap;
//use std::rc::Rc;

use crate::commands::execution::{ValueExecution, ValueResult};
use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
//use super::namespace;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
use crate::PicoValue;

pub type LookupDict = HashMap<String, PicoValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupTable {
    pub entries: LookupDict,
    pub default: PicoValue,

    // namespaces this lookup table is available in
    pub namespaces: Option<Vec<String>>,
}

impl Default for LookupTable {
    fn default() -> Self {
        Self {
            default: PicoValue::String("unknown".to_string()),
            entries: HashMap::new(),
            namespaces: None,
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
