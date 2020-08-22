use crate::rules::PicoRules;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::context::PicoContext;

#[derive(Debug)]
pub struct PicoRuntime<'a> {
  pub variables: Vec<HashMap<String, String>>,
  pub json_variables: Vec<HashMap<String, JsonValue>>,
  current_rules: Vec<&'a PicoRules>,
  root_rule: &'a PicoRules, // borrowed reference of the top level rulefile
}
impl<'a> PicoRuntime<'a> {
  pub fn new(root_rule: &'a PicoRules) -> Self {
    Self {
      variables: Vec::new(),
      json_variables: Vec::new(),
      current_rules: Vec::new(),
      root_rule,
    }
  }

  pub fn exec_root_with_context(&mut self, ctx: &mut PicoContext) {
    self.root_rule.run_with_context(self, ctx)
  }

  /*
   * Maybe...
   * when switching to an included file,
   *  push the include onto current_rules
   *   exec the last current_rules
   *  pop off current rules
   */
  pub fn exec_current_with_context(&mut self, ctx: &mut PicoContext) {
    self
      .current_rules
      .last_mut()
      .unwrap()
      .run_with_context(self, ctx);
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
}
