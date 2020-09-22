use serde::{Deserialize, Serialize};
use std::collections::HashMap;
//use std::rc::Rc;
use std::fs::File;

use crate::PicoValue;

pub type LookupDict = HashMap<String, PicoValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupTable {
    pub entries: LookupDict,
    pub default: PicoValue,

    // namespaces this lookup table is available in
    pub namespaces: Option<Vec<String>>,
}

impl Default for LookupTable {
    fn default() -> Self {
        Self {
            default: PicoValue::String("unknown".to_string()),
            entries: HashMap::new(),
            namespaces: None,
        }
    }
}

impl LookupTable {
    pub fn new() -> LookupTable {
        Default::default()
    }

    pub fn lookup(&self, key: &str) -> &PicoValue {
        if let Some(value) = self.entries.get(key) {
            value
        } else {
            &self.default
        }
    }
}

//pub type Lookups = HashMap<String, Rc<LookupTable>>;
pub type Lookups = HashMap<String, LookupType>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum LookupType {
    ExternalTable(String),
    InternalTable(LookupTable),
}

pub fn get_external_lookup_names(lookups: &Lookups) -> Vec<(&String, &String)> {
    let c: Vec<(&String, &String)> = lookups
        .iter()
        .filter_map(|(name, typ)| match typ {
            LookupType::ExternalTable(t) => Some((name, t)),
            _ => None,
        })
        .collect();

    debug!("ccc1 {:?}", c);
    c
}

pub fn load_into_cache(filename: &str, cache: &mut HashMap<String, LookupTable>) {
    let k = filename.to_string();

    match File::open(filename) {
        Ok(opened_file) => {
            let lookup_file: LookupTable = serde_json::from_reader(opened_file).unwrap();
            cache.insert(k, lookup_file);
        }
        Err(x) => {
            error!("Failed to open: {:?}", x);
        }
    }
}
