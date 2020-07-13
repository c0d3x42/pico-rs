use anyhow::{anyhow, Context, Result};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::command::RuleFile;
use crate::command::{Execution, ExecutionResult, FnResult, RuleFileRoot};
use crate::context::{PicoContext, PicoState};
use crate::errors::PicoError;
use crate::lookups::LookupTable;
use crate::values::PicoValue;

pub struct IncludedFileSeed<T = RuleFile> {
    lookups: HashMap<String, T>,
}

impl IncludedFileSeed {
    fn new() -> Self {
        IncludedFileSeed {
            lookups: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct IncludeFileDriver {
    filename: String,
}

impl<'de> Deserialize<'de> for IncludeFileDriver {
    //type Value = IncludeFileDriver;

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // use serde::de::Error;
        debug!("deserializing.1 ");
        let s = String::deserialize(deserializer)?;
        debug!("deserializing {:?}", s);

        //let nf: RuleFile = serde_json::from_reader(File::open(&s).unwrap()).unwrap();
        //debug!("NEW RULE FILE {:?}", nf);

        Ok(IncludeFileDriver { filename: s })
    }
}

impl Serialize for IncludeFileDriver {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // FIXME: needs to write out the self.rule to self._filename
        serializer.serialize_str(&self.filename)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludeFile {
    include: IncludeFileDriver,
}

impl Execution for IncludeFile {
    fn name(&self) -> String {
        format!("include [{:?}]", self.include).to_string()
    }

    fn run_with_context(&self, state: &mut PicoState, ctx: &mut PicoContext) -> FnResult {
        info!("running included module");
        //self.include.rule.run_with_context(state, ctx)
        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}

#[derive(Debug)]
pub enum LoadResult {
    Ok,
    NoSuchFile,
    ParseError,
    UnknownError,
}

#[derive(Debug)]
pub struct LoadedFile<T> {
    result: LoadResult,
    content: Option<T>,
}

pub type LoadedCache<T> = HashMap<String, LoadedFile<T>>;

pub enum LoaderError {
    NoSuchFile,
    ParseFailure,
}

/*
pub fn file_loader<T>( filename: &str ) -> Result<T, LoaderError>{

    let file =  File::open(filename).

}
*/

pub fn load_file(
    filename: &str,
    cache: &mut LoadedCache<RuleFile>,
    lookup_cache: &mut HashMap<String, Rc<LookupTable>>,
) -> FnResult {
    if cache.contains_key(filename) {
        warn!("circular filename: {}", filename);
        return Ok(ExecutionResult::Continue(PicoValue::Boolean(true)));
    }

    // errors with anyhow:error
    let f = File::open(filename).context(format!("CANT READ {}", filename))?;

    let nf: RuleFile = serde_json::from_reader(f)?;

    // find all root level include directives
    let include_filenames: Vec<&String> = nf
        .root
        .iter()
        .filter_map(|v| match v {
            RuleFileRoot::IncludeFile(f) => Some(&f.include.filename),
            _ => None,
        })
        .collect();

    // load each lookup table into the cache
    let lookup_iter = nf.lookups.iter();
    for (key, table) in lookup_iter {
        lookup_cache.insert(key.to_string(), Rc::clone(table));
    }

    println!("values: {:?}", include_filenames);
    for ifilename in include_filenames {
        if ifilename != filename {
            warn!("attempt to include self again");
            match load_file(ifilename, cache, lookup_cache) {
                Ok(_) => {}
                Err(e) => {
                    warn!("INSERT error {:?}", e);
                    cache.insert(
                        String::from(ifilename),
                        LoadedFile {
                            result: LoadResult::NoSuchFile,
                            content: None,
                        },
                    );
                }
            }
        }
    }

    let lf: LoadedFile<RuleFile> = LoadedFile {
        result: LoadResult::Ok,
        content: Some(nf),
    };

    cache.insert(String::from(filename), lf);

    Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
}

pub type LookupName = String;

pub fn populate_lookups(file_cache: &LoadedCache<RuleFile>) -> HashMap<String, Rc<LookupTable>> {
    let lookups: Vec<_> = file_cache
        .values()
        .filter_map(|loaded_file| match &loaded_file.content {
            Some(c) => Some(&c.lookups),
            None => None,
        })
        .collect();

    println!("lookups: {:?}", lookups);

    let mut hm: HashMap<String, Rc<LookupTable>> = HashMap::new();

    for (_filename, loaded_file) in file_cache {
        if let Some(rule_file) = &loaded_file.content {
            for (lookup_name, lookup_table) in rule_file.lookups.iter() {
                hm.insert(lookup_name.to_string(), Rc::clone(lookup_table));
            }
        }
    }

    return hm;
}

pub struct PicoRules {
    rule_cache: LoadedCache<RuleFile>,
    lookup_cache: HashMap<String, Rc<LookupTable>>,
    entrypoint: String,
}

impl PicoRules {
    pub fn new(filename: &str) -> Self {
        let mut rule_cache = HashMap::new();
        let mut lookup_cache = HashMap::new();

        // load the initial rule file
        //load_file(filename, &mut rule_cache, &mut lookup_cache);

        // populate the lookup tables
        //let lookup_cache = populate_lookups(&rule_cache);

        // let ps = PicoState::new(&lookups);

        PicoRules {
            rule_cache,
            lookup_cache,
            entrypoint: String::from(filename),
        }
    }

    pub fn load(&mut self) -> FnResult {
        load_file(
            &self.entrypoint,
            &mut self.rule_cache,
            &mut self.lookup_cache,
        )?;
        self.lookup_cache = populate_lookups(&mut self.rule_cache);

        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }

    pub fn make_state(&self) -> PicoState {
        let ps = PicoState::new(&self.lookup_cache);
        return ps;
    }

    pub fn build(&mut self) -> &Self {
        self
    }

    pub fn run_with_context(&self, state: &mut PicoState, context: &mut PicoContext) {
        let loaded_file = self.rule_cache.get(&self.entrypoint).unwrap();

        loaded_file
            .content
            .as_ref()
            .unwrap()
            .run_with_context(state, context)
            .expect("something");
        return;
    }
}
