use crate::lookups::Lookups;
use crate::PicoValue;
use serde_json::Value;

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub enum StateValue {
    Boolean(bool),
    Number(isize),
    String(String),
}

pub type VariablesMap = HashMap<String, PicoValue>;

#[derive(Serialize, Debug)]
pub struct PicoContext {
    pub variables: VariablesMap,
    pub local_variables: VariablesMap,
    pub lookup_tables: Lookups,
    pub json: Option<serde_json::Value>,
}

impl Default for PicoContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            local_variables: HashMap::new(),
            lookup_tables: HashMap::new(),
            json: None,
        }
    }
}

impl PicoContext {
    pub fn new() -> PicoContext {
        Default::default()
    }

    pub fn set_json(mut self, json: serde_json::Value) -> Self {
        self.json = Some(json);
        self
    }

    pub fn set_value(&mut self, key: &str, value: PicoValue) {
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
