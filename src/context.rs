use crate::include::LoadedRuleMap;
use crate::lookups::{LookupTable, Lookups};
use crate::PicoValue;

use serde::Serialize;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub enum StateValue {
    Boolean(bool),
    Number(isize),
    String(String),
}

pub type VariablesMap = HashMap<String, PicoValue>;

#[derive(Serialize)]
pub struct PicoContext {
    pub variables: VariablesMap,
    pub local_variables: VariablesMap,
    pub lookup_tables: Lookups,
}

impl PicoContext {
    pub fn new() -> PicoContext {
        PicoContext {
            variables: HashMap::new(),
            local_variables: HashMap::new(),
            lookup_tables: HashMap::new(),
        }
    }
    pub fn set_value(&mut self, key: &str, value: PicoValue) -> () {
        self.local_variables.insert(key.to_string(), value);
    }

    pub fn get_value(&self, key: &str) -> Option<&PicoValue> {
        if let Some(plv) = self.local_variables.get(key) {
            return Some(plv);
        } else if let Some(pv) = self.variables.get(key) {
            return Some(pv);
        }
        None
    }
}

pub type PicoHashMap = HashMap<String, String>;
