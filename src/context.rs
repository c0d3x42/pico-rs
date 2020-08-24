use crate::lookups::Lookups;
use crate::PicoValue;

use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub enum StateValue {
    Boolean(bool),
    Number(isize),
    String(String),
}

type Namespace = String;
pub type VariablesMap = HashMap<String, PicoValue>;
type NamespaceVariableMap = HashMap<Namespace, VariablesMap>;

#[derive(Serialize, Debug)]
pub struct PicoContext {
    pub namespaced_variables: NamespaceVariableMap,
    pub variables: VariablesMap,
    pub local_variables: VariablesMap,
    pub lookup_tables: Lookups,
    pub input_json: Option<serde_json::Value>,
}

impl Default for PicoContext {
    fn default() -> Self {
        Self {
            namespaced_variables: HashMap::new(),
            variables: HashMap::new(),
            local_variables: HashMap::new(),
            lookup_tables: HashMap::new(),
            input_json: None,
        }
    }
}

impl PicoContext {
    pub fn new() -> PicoContext {
        Default::default()
    }

    pub fn set_json(mut self, json: serde_json::Value) -> Self {
        self.input_json = Some(json);
        self
    }

    pub fn ns_add(&mut self, ns: &str) {
        let r = self
            .namespaced_variables
            .insert(ns.to_string(), HashMap::new());
        if let Some(original) = r {
            warn!("overwritten namespace {}", ns);
            trace!(" original: {:?}", original);
        }
    }

    pub fn ns_del(&mut self, ns: &str) {
        let r = self.namespaced_variables.remove(ns);
        if let Some(original) = r {
            info!("removed namespace {}", ns);
            trace!(" original: {:?}", original);
        }
    }

    pub fn ns_get(&self, ns: &str, key: &str) -> Option<&PicoValue> {
        self.namespaced_variables.get(ns).and_then(|hm| hm.get(key))
    }

    pub fn ns_set(&mut self, ns: &str, key: &str, value: &PicoValue) {
        self.namespaced_variables
            .get_mut(ns)
            .and_then(|hm| hm.insert(key.to_string(), value.clone()));
    }

    pub fn local_set(&mut self, key: &str, value: &PicoValue) {
        self.local_variables.insert(key.to_string(), value.clone());
    }

    pub fn local_get(&self, key: &str) -> Option<&PicoValue> {
        self.local_variables.get(key)
    }

    pub fn local_clear(&mut self) {
        self.local_variables.clear()
    }

    pub fn get_value(&self, key: &str) -> Option<&PicoValue> {
        match self.local_get(key) {
            Some(v) => Some(v),
            None => {
                if let Some(input_json) = &self.input_json {
                    trace!("Looking for key [{}] in input json", key);
                    let json_path = format!("/{}", key);
                    input_json.pointer(&json_path)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_final_ctx(&mut self) -> &VariablesMap {
        self.variables
            .insert("input".to_string(), json!(&self.input_json));
        self.variables
            .insert("locals".to_string(), json!(&self.local_variables));
        self.variables
            .insert("namespaced".to_string(), json!(&self.namespaced_variables));

        &self.variables
    }
}

pub type PicoHashMap = HashMap<String, String>;
