use crate::errors::PicoError;
use crate::lookups::{LookupTable, Lookups};
use crate::PicoValue;

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub enum StateValue {
  Boolean(bool),
  Number(isize),
  String(String),
}

pub type VariablesMap = HashMap<String, PicoValue>;

#[derive(Serialize)]
pub struct Context {
  pub variables: VariablesMap,
  pub state: HashMap<String, StateValue>,
  pub local_variables: VariablesMap,
  pub lookup_tables: Lookups,
}

impl Context {
  pub fn new() -> Context {
    Context {
      state: HashMap::new(),
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

  pub fn add_table(&mut self, table_name: &String, lookups: &LookupTable) {
    let lookup_table = self.lookup_tables.get(table_name);

    //self.lookup_tables.insert(table_name.to_string(), lookups.clone());
  }

  pub fn lookup(&self, table_name: &String, entry_name: &String) -> Option<&PicoValue> {
    if let Some(table) = self.lookup_tables.get(table_name) {
      let t = table.lookup(entry_name);

      return Some(t);
    }
    return None;
  }
}

pub struct PicoState {
  lookup_tables: HashMap<String, LookupTable>,
}

impl PicoState {
  pub fn new() -> Self {
    Self {
      lookup_tables: HashMap::new(),
    }
  }

  pub fn add(&self, table_name: &String, table: &LookupTable) {
    let mut i = LookupTable::new();

    for (key, value) in &table.entries {
      i.entries.insert(key.to_string(), *value);
    }
  }
}

pub type PicoHashMap = HashMap<String, String>;

#[derive(Debug)]
pub struct PicoContext {
  //pub values: HashMap<String,String>
  pub values: PicoHashMap,
}
impl PicoContext {
  pub fn new() -> Self {
    let mut t = PicoHashMap::new();
    t.insert("lop".to_string(), "LOP".to_string());
    info!("New PicoContext");
    Self { values: t }
  }

  pub fn get(&self, name: &str) -> Option<&String> {
    return self.values.get(name);
  }

  pub fn put(&mut self, key: &str, value: &str) {
    self.values.insert(key.to_string(), value.to_string());
  }
}
