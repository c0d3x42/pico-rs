use crate::command::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum StateValue {
  Boolean(bool),
  Number(isize),
  String(String),
}

pub type VariablesMap = HashMap<String, Value>;

pub struct Context {
  pub variables: VariablesMap,
  pub state: HashMap<String, StateValue>,
}

impl Context {
  pub fn new() -> Self {
    Self {
      state: HashMap::new(),
      variables: HashMap::new(),
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
