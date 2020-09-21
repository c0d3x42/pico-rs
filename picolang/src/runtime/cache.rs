use crate::errors::RuleFileError;
use crate::rules::{
  lookups::{load_into_cache, LookupTable},
  PicoRules,
};
use crate::values::PicoValue;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt;

/// LookupCache: lookup tables that are shared between PicoRules
#[derive(Debug)]
pub struct LookupCache {
  // key: filename,
  // value: lookup table
  cache: HashMap<String, LookupTable>,
}
impl Default for LookupCache {
  fn default() -> Self {
    Self {
      cache: HashMap::new(),
    }
  }
}
impl fmt::Display for LookupCache {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "LookupCache: [{}]", self.cache.keys().join(", "))
  }
}
impl LookupCache {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn load(&mut self, lookup_filename: &str) -> Result<(), RuleFileError> {
    if self.cache.contains_key(lookup_filename) {
      info!("Lookup cache already has {}", lookup_filename);
    } else {
      load_into_cache(lookup_filename, &mut self.cache);
    }

    Ok(())
  }

  pub fn lookup(&self, lookup_filename: &str, key: &str) -> Option<&PicoValue> {
    self
      .cache
      .get(lookup_filename)
      .and_then(|t| Some(t.lookup(key)))
  }
}

#[derive(Debug)]
pub struct PicoRulesCache {
  cache: HashMap<String, PicoRules>,
  including_paths: HashMap<String, Vec<String>>,
}

impl Default for PicoRulesCache {
  fn default() -> Self {
    Self {
      cache: HashMap::new(),
      including_paths: HashMap::new(),
    }
  }
}

impl fmt::Display for PicoRulesCache {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "PicoRulesCache: [{}]", self.cache.keys().join(", "))
  }
}

impl PicoRulesCache {
  pub fn new() -> Self {
    Default::default()
  }
  pub fn get(&self, name: &str) -> Option<&PicoRules> {
    self.cache.get(name)
  }

  pub fn values(&self) -> std::collections::hash_map::Values<String, PicoRules> {
    self.cache.values()
  }

  pub fn filenames(&self) -> Vec<String> {
    self.cache.keys().cloned().collect()
  }

  pub fn store(&mut self, filename: &str, pico_rule: PicoRules) -> Result<(), RuleFileError> {
    // FIXME detect recursive includes
    self.cache.insert(filename.to_string(), pico_rule);

    Ok(())
  }

  pub fn load(&mut self, entry_filename: &str) -> Result<(), RuleFileError> {
    if self.cache.contains_key(entry_filename) {
      info!("already have {}", entry_filename);
    } else {
      debug!("Attempting to load {}", entry_filename);
      PicoRules::load_into_cache(entry_filename, &mut self.cache);
    }
    Ok(())
  }
}
