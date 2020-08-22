use crate::rules::PicoRules;
use std::collections::HashMap;

use crate::context::PicoContext;

pub struct PicoRuntime<'a> {
  pub variables: Vec<HashMap<String, String>>,
  current_rules: Vec<&'a PicoRules>,
  root_rule: &'a PicoRules, // borrowed reference of the top level rulefile
}
impl<'a> PicoRuntime<'a> {
  pub fn new(root_rule: &'a PicoRules) -> Self {
    Self {
      variables: Vec::new(),
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
    self.variables.push(HashMap::new())
  }

  pub fn remove(&mut self) {
    self.variables.pop();
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
