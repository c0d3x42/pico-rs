use crate::lookups::{LookupTable, Lookups};
use crate::PicoValue;

use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

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
  pub local_variables: VariablesMap,
  pub lookup_tables: Lookups,
}

impl Context {
  pub fn new() -> Context {
    Context {
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
  pub lookup_tables: &'a HashMap<String, LookupTable>,
  pub branch_hits: HashMap<Uuid, u64>,
}

impl<'a> PicoState<'a> {
  pub fn new(plookups: &'a HashMap<String, LookupTable>) -> Self {
    Self {
      lookup_tables: plookups,
      branch_hits: HashMap::new(),
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
