use crate::rules::PicoRules;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::context::PicoContext;
use crate::rules::lookups::LookupTable;
use crate::values::PicoValue;

type Namespace = String;
type VariableMap = HashMap<String, PicoValue>;

#[derive(Debug)]
pub struct PicoRuntime<'a> {
  pub globals: HashMap<String, PicoValue>,
  pub namespaced_variables: HashMap<Namespace, VariableMap>,

  pub namespaced_lookups: HashMap<(&'a str, &'a str), &'a LookupTable>,

  feature_globals_readonly: bool,
  feature_namespaces: bool,
  pub root_rule: PicoRules, // moved reference of the top level rulefile
}
impl<'a> PicoRuntime<'a> {
  pub fn new(root_rule: PicoRules) -> Self {
    Self {
      globals: HashMap::new(),
      namespaced_variables: HashMap::new(), // all namespaced variables
      namespaced_lookups: HashMap::new(),
      /// readonly globals by default
      feature_globals_readonly: true,
      /// enabled by default
      feature_namespaces: true,
      root_rule,
    }
  }

  pub fn enable_mutable_globals(mut self) -> Self {
    self.feature_globals_readonly = false;
    self
  }

  pub fn disable_namespaces(mut self) -> Self {
    self.feature_namespaces = false;
    self
  }

  /// builder to add a global value
  pub fn add_global(mut self, key: &str, value: &PicoValue) -> Self {
    self.global_set(key, value);
    self
  }

  pub fn initialise(mut self) -> Self {
    info!("xx");

    let mut namespaces: Vec<String> = Vec::new();
    self.root_rule.all_namespace(&mut namespaces);

    info!("ALL NAMESPACES {}", namespaces.join(","));

    // register all declared namespaces
    for ns in namespaces {
      self.add_namespace(&ns);
    }

    //let mut cache_map: HashMap<(&str, &str), &LookupTable> = HashMap::new();
    //self.root_rule.all_namespaced_lookup_tables(&mut cache_map);

    // info!("CACHE MAP after {:?}", cache_map);
    //self.namespaced_lookups = cache_map;

    self
  }

  pub fn make_ctx(&self) -> PicoContext {
    let mut pc = PicoContext::new();
    for ns in self.namespaced_variables.keys() {
      pc.ns_add(ns);
    }
    pc
  }

  pub fn exec_root_with_context(&self, ctx: &mut PicoContext) {
    self.root_rule.run_with_context(self, ctx)
  }

  pub fn global_get(&self, key: &str) -> Option<&PicoValue> {
    self.globals.get(key)
  }

  pub fn global_set(&mut self, key: &str, value: &PicoValue) {
    if self.feature_globals_readonly {
      warn!(
        "Global values are imutable, attempted to set {} = {}",
        key, value
      );
    } else {
      self.globals.insert(key.to_string(), value.clone());
    }
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
