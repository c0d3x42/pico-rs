use crate::rules::PicoRules;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::context::PicoContext;
use crate::values::PicoValue;

type Namespace = String;
type VariableMap = HashMap<String, JsonValue>;

#[derive(Debug)]
pub struct PicoRuntime<'a> {
  pub variables: Vec<HashMap<String, String>>,
  pub globals: HashMap<String, PicoValue>,

  pub namespaced_variables: HashMap<Namespace, VariableMap>,

  pub json_variables: Vec<HashMap<String, JsonValue>>,
  root_rule: &'a PicoRules, // borrowed reference of the top level rulefile
}
impl<'a> PicoRuntime<'a> {
  pub fn new(root_rule: &'a PicoRules) -> Self {
    Self {
      variables: Vec::new(),
      globals: HashMap::new(),
      namespaced_variables: HashMap::new(), // all namespaced variables
      json_variables: Vec::new(),
      root_rule,
    }
  }

  pub fn initialise(mut self) -> Self {
    info!("xx");

    let mut namespaces: Vec<String> = Vec::new();
    self.root_rule.all_namespace(&mut namespaces);

    info!("ALL NAMESPACES {}", namespaces.join(","));

    for ns in namespaces {
      self.add_namespace(&ns);
    }

    self
  }

  pub fn make_ctx(&self) -> PicoContext {
    let mut pc = PicoContext::new();
    for ns in self.namespaced_variables.keys() {
      pc.ns_add(ns);
    }
    pc
  }

  pub fn exec_root_with_context(&mut self, ctx: &mut PicoContext) {
    self.root_rule.run_with_context(self, ctx)
  }

  pub fn add(&mut self) {
    info!("runtimes: {}", self.variables.len());
    self.variables.push(HashMap::new());
    self.json_variables.push(HashMap::new());
  }

  pub fn remove(&mut self) {
    self.variables.pop();
  }

  pub fn json_get(&self, key: &str) -> Option<&serde_json::Value> {
    for variable in self.json_variables.iter() {
      if let Some(j) = variable.get(key) {
        return Some(j);
      }
    }
    None
  }

  pub fn json_set(&mut self, key: &str, value: &serde_json::Value) {
    if let Some(hm) = self.json_variables.last_mut() {
      hm.insert(key.to_string(), value.clone());
    }
  }

  pub fn json_pop(&mut self) -> HashMap<String, JsonValue> {
    match self.json_variables.pop() {
      Some(hm) => hm,
      None => HashMap::new(),
    }
  }

  pub fn get(&self, key: &str) -> Option<&String> {
    for variable in self.variables.iter() {
      match variable.get(key) {
        Some(v) => return Some(v),
        None => continue,
      }
    }
    None
  }

  pub fn set(&mut self, key: &str, value: &str) {
    if let Some(mut hm) = self.variables.pop() {
      hm.insert(key.to_string(), value.to_string());
      self.variables.push(hm);
    }
  }

  pub fn global_get(&self, key: &str) -> Option<&PicoValue> {
    self.globals.get(key)
  }

  pub fn global_set(&mut self, key: &str, value: &PicoValue) {
    self.globals.insert(key.to_string(), value.clone());
  }

  pub fn new_namespace(&mut self, name: &str) {
    self
      .namespaced_variables
      .insert(name.to_string(), HashMap::new());
  }

  pub fn add_namespace(&mut self, name: &str) {
    if !self.namespaced_variables.contains_key(name) {
      self.new_namespace(name);
    } else {
      warn!("Attempt to redeclare namespace: [{}]", name);
    }
  }

  pub fn ns_get(&self, ns: &str, key: &str) -> Option<&PicoValue> {
    self.namespaced_variables.get(ns).and_then(|hm| hm.get(key))
  }

  pub fn ns_set(&mut self, ns: &str, key: &str, value: &PicoValue) {
    if let Some(ns_map) = self.namespaced_variables.get_mut(ns) {
      ns_map.insert(key.to_string(), value.clone());
    } else {
      warn!(
        "namespace {} does not exist, can not save {} = {}",
        ns, key, value
      );
    }
  }
}
