use crate::errors::RuleFileError;
use std::collections::HashMap;
use std::{env, fmt, path::Path};
use std::{fs, io};

use crate::context::PicoContext;
use crate::rules::lookups::LookupTable;
use crate::values::PicoValue;

type Namespace = String;
type VariableMap = HashMap<String, PicoValue>;

mod cache;
use cache::{LookupCache, PicoRulesCache};

#[derive(Debug)]
pub struct PicoRuntime<'a> {
    pub globals: HashMap<String, PicoValue>,
    pub namespaced_variables: HashMap<Namespace, VariableMap>,

    pub namespaced_lookups: HashMap<(&'a str, &'a str), &'a LookupTable>,

    feature_globals_readonly: bool,
    feature_namespaces: bool,

    rules_directory: String,
    rules_cache: PicoRulesCache,
    default_rule_name: String,

    lookup_cache: LookupCache,
}
impl<'a> PicoRuntime<'a> {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            namespaced_variables: HashMap::new(), // all namespaced variables
            namespaced_lookups: HashMap::new(),
            /// readonly globals by default
            feature_globals_readonly: true,
            /// enabled by default
            feature_namespaces: true,
            rules_directory: String::from("rules/"),
            rules_cache: PicoRulesCache::new(),
            default_rule_name: String::from("pico.rule.json"),
            lookup_cache: LookupCache::new(),
        }
    }

    pub fn load_rules(&mut self) -> Result<(), RuleFileError> {
        let path = Path::new(&self.rules_directory);
        env::set_current_dir(path)?;

        for entry in fs::read_dir(".")? {
            let entry = entry?;
            let path = entry.path();
            info!("FILENAME: {:?}", path.file_name());

            if let Some(p) = path.file_name() {
                if let Some(pp) = p.to_str() {
                    if pp.ends_with(".lookup.json") {
                        self.lookup_cache.load(pp)?;
                    } else if pp.ends_with(".rule.json") {
                        self.rules_cache.load(pp)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn set_default_rule(mut self, filename: &str) -> Self {
        self.default_rule_name = String::from(filename);
        self
    }

    pub fn set_rules_directory(mut self, directory: &str) -> Self {
        self.rules_directory = String::from(directory);
        self
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

    pub fn load_lookups(&mut self) {
        for value in self.rules_cache.values() {
            for (table_name, file_name) in value.external_lookups() {
                debug!("Loading external lookup from {}", file_name);
                self.lookup_cache.load(file_name);
            }
        }
    }

    pub fn initialise(mut self) -> Self {
        info!("xx");

        self.load_rules();
        self.load_lookups();

        let mut namespaces: Vec<String> = Vec::new();
        //self.root_rule.all_namespace(&mut namespaces);

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

    pub fn rule_file_names(&self) -> Vec<String> {
        self.rules_cache.filenames()
    }

    pub fn make_ctx(&self) -> PicoContext {
        let mut pc = PicoContext::new();
        for ns in self.namespaced_variables.keys() {
            pc.ns_add(ns);
        }
        pc
    }

    pub fn exec_rule_with_context(&self, rule_name: &str, ctx: &mut PicoContext) {
        if let Some(ref pico_rule) = self.rules_cache.get(rule_name) {
            pico_rule.run_with_context(self, ctx)
        }
    }

    pub fn exec_root_with_context(&self, ctx: &mut PicoContext) {
        info!("Running with default rule: {}", self.default_rule_name);
        self.exec_rule_with_context(&self.default_rule_name, ctx)
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
        info!("NAMESPACES {:?}", self.namespaced_variables);
        self.namespaced_variables
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

    pub fn table_lookup(&self, table_filename: &str, key: &str) -> Option<&PicoValue> {
        self.lookup_cache.lookup(table_filename, key)
    }
}