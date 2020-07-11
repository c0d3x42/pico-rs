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
use crate::context::{Context, PicoState};
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

    fn run_with_context(&self, state: &mut PicoState, ctx: &mut Context) -> FnResult {
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

pub fn load_file(
    filename: &str,
    cache: &mut LoadedCache<RuleFile>,
    lookup_cache: &mut HashMap<String, Rc<LookupTable>>,
) {
    if cache.contains_key(filename) {
        warn!("circular filename: {}", filename);
        return;
    }

    let f = File::open(filename);
    if f.is_err() {
        cache.insert(
            String::from(filename),
            LoadedFile {
                result: LoadResult::NoSuchFile,
                content: None,
            },
        );
        warn!("failed to open file {}", filename);
        return;
    }

    let nf: RuleFile = serde_json::from_reader(f.unwrap()).unwrap();
    //let nf: RuleFile = serde_json::from_reader(File::open(filename).unwrap()).unwrap();

    let include_filenames: Vec<&String> = nf
        .root
        .iter()
        .filter_map(|v| match v {
            RuleFileRoot::IncludeFile(f) => Some(&f.include.filename),
            _ => None,
        })
        .collect();

    let lookup_iter = nf.lookups.iter();
    for (key, table) in lookup_iter {
        lookup_cache.insert(key.to_string(), Rc::clone(table));
    }

    println!("values: {:?}", include_filenames);
    for ifilename in include_filenames {
        if ifilename != filename {
            warn!("attempt to include self again");
            load_file(ifilename, cache, lookup_cache);
        }
    }

    let lf: LoadedFile<RuleFile> = LoadedFile {
        result: LoadResult::Ok,
        content: Some(nf),
    };

    cache.insert(String::from(filename), lf);
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

    for (filename, loaded_file) in file_cache {
        if let Some(rule_file) = &loaded_file.content {
            for (lookup_name, lookup_table) in rule_file.lookups.iter() {
                hm.insert(lookup_name.to_string(), Rc::clone(lookup_table));
            }
        }
    }

    return hm;
    /*
    let lookups_with_values: Vec<_> = lookups.iter().filter_map(|v| v.as_ref()).collect();

    println!("lookups values: {:?}", lookups_with_values);

    let mut all_lookups: HashMap<String, &LookupTable> = HashMap::new();
    for hm in lookups_with_values {
        for (key, value) in hm {
            all_lookups.insert(String::from(key), value);
        }
    }
    all_lookups
    */
}

pub struct PicoRules {
    rule_cache: LoadedCache<RuleFile>,
    entrypoint: String,
}

impl PicoRules {
    pub fn new(filename: &str) -> Self {
        let mut rule_cache = HashMap::new();
        let mut lookup_cache = HashMap::new();

        // load the initial rule file
        load_file(filename, &mut rule_cache, &mut lookup_cache);

        let lookups = populate_lookups(&rule_cache);

        // let ps = PicoState::new(&lookups);

        PicoRules {
            rule_cache,
            entrypoint: String::from(filename),
        }
    }

    pub fn build(&mut self) -> &Self {
        self
    }

    pub fn run_with_context(&self, context: &mut Context) {
        let loaded_file = self.rule_cache.get(&self.entrypoint).unwrap();

        /*
        let mut ps = PicoState::new(&self.lookups);
        loaded_file
            .content
            .as_ref()
            .unwrap()
            .run_with_context(&mut ps, context);

        */
        return;
    }
}
