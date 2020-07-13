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

#[derive(Debug)]
pub struct PicoState<'a> {
    pub branch_hits: HashMap<Uuid, u64>,
    pub lookup_cache: &'a HashMap<String, Rc<LookupTable>>,
}

impl<'a> PicoState<'a> {
    pub fn new(lookups: &'a HashMap<String, Rc<LookupTable>>) -> Self {
        Self {
            branch_hits: HashMap::new(),
            lookup_cache: lookups,
        }
    }

    pub fn get_lookup_value(&self, table_name: &str, table_key: &str) -> Option<&PicoValue> {
        if let Some(lookup_table) = self.lookup_cache.get(table_name) {
            Some(lookup_table.lookup(&table_key.to_string()))
        } else {
            None
        }
    }

    pub fn increment_branch_hit(&mut self, uuid: &Uuid) {
        if let Some(v) = self.branch_hits.get_mut(uuid) {
            *v += 1;
        } else {
            self.branch_hits.insert(uuid.clone(), 1);
        }
    }
}

pub type PicoHashMap = HashMap<String, String>;
