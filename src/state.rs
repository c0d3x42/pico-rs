use crate::include::LoadedRuleMap;
use crate::lookups::LookupTable;
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

#[derive(Debug)]
pub struct PicoState<'a> {
    pub branch_hits: HashMap<Uuid, u64>,
    pub lookup_cache: HashMap<String, LookupTable>,
    pub current_include_path: Vec<String>,
    pub rulefile_cache: &'a LoadedRuleMap,
}

impl<'a> PicoState<'a> {
    pub fn new(
        //lookups: &'a HashMap<String, Rc<LookupTable>>,
        rulefile_cache: &'a LoadedRuleMap,
        root_file: &str,
    ) -> Self {
        Self {
            branch_hits: HashMap::new(),
            lookup_cache: HashMap::new(),
            current_include_path: vec![String::from(root_file)],
            rulefile_cache,
        }
    }

    pub fn get_include_path(&self) -> String {
        self.current_include_path.join(" -> ")
    }

    pub fn put_include_path(&mut self, filename: &str) {
        self.current_include_path.push(String::from(filename));
    }

    pub fn pop_include_path(&mut self) {
        self.current_include_path.pop();
    }

    pub fn get_lookup_value(&self, lookup_table: &str, lookup_key: &str) -> Option<&PicoValue> {
        /*
        let mut compounded_key: String = String::from(included_file_name);
        compounded_key.push('/');
        compounded_key.push_str(lookup_table);

        trace!("Looking up compound value: {}", compounded_key);

        self.lookup_cache.get(&compounded_key)
        */
        trace!("available lookup file {:?}", self.current_include_path);

        for included_filename in self.current_include_path.iter().rev() {
            trace!(
                "Looking up in [{}] {}/{}",
                included_filename,
                lookup_table,
                lookup_key
            );
            if let Some(rf) = self.rulefile_cache.get(included_filename) {
                if let Some(content) = &rf.content {
                    if let Some(table) = content.lookups.get(lookup_table) {
                        let option_value = table.entries.get(lookup_key);

                        let found_value = option_value.unwrap_or(&table.default);
                        debug!(
                            "Found look in [{}] for {}/{}",
                            included_filename, lookup_table, lookup_key
                        );
                        return Some(found_value);
                    }
                    debug!(
                        "Failed to find in [{}] {}/{}",
                        included_filename, lookup_table, lookup_key
                    );
                }
            }
        }

        None
    }

    pub fn get_lookup_value_old(&self, table_name: &str, table_key: &str) -> Option<&PicoValue> {
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
