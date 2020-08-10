use anyhow::{anyhow, Context, Result};
use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use std::collections::{HashMap, HashSet};
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
        //return Err(serde::de::Error::custom(format!("aaaarg")));
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
        info!("running included module {}", self.include.filename);

        let rf_result = state.rulefile_cache.get(&self.include.filename);

        match rf_result {
            Some(rf) => {
                rf.run_with_context(state, ctx);
            }
            None => {}
        }
        //self.include.rule.run_with_context(state, ctx)
        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}

#[derive(Debug)]
pub enum LoadResult {
    Ok,
    NotLoaded,
    NoSuchFile,
    ParseError,
    UnknownError,
}

pub type LoadedRuleMap = HashMap<String, LoadedRuleFile>;

#[derive(Debug)]
pub struct LoadedRuleFile {
    filename: String,
    include_path: Vec<String>,
    result: LoadResult,
    content: Option<RuleFile>,
    include_set: HashSet<String>,
}

impl LoadedRuleFile {
    pub fn new(filename: &str, parent_path: &Vec<String>) -> Self {
        let mut include_path: Vec<String> = parent_path.iter().map(|s| String::from(s)).collect();
        include_path.push(String::from(filename));

        // maybe dont need this, its duplicating whats in the Vec
        let mut include_set: HashSet<&str> = HashSet::new();
        for i in include_path.iter() {
            include_set.insert(i);
        }

        LoadedRuleFile {
            filename: String::from(filename),
            include_path,
            result: LoadResult::NotLoaded,
            content: None,
            include_set: HashSet::new(),
        }
    }

    fn included_filenames(&self) -> Vec<String> {
        match &self.content {
            None => Vec::new(),
            Some(content) => {
                let included_filenames: Vec<String> = content
                    .root
                    .iter()
                    .filter_map(|v| match v {
                        RuleFileRoot::IncludeFile(f) => Some(f.include.filename.clone()),
                        _ => None,
                    })
                    .collect();
                return included_filenames;
            }
        }
    }

    pub fn load(mut self, root_cache: &mut LoadedRuleMap) -> Result<LoadResult> {
        let f = File::open(&self.filename).context(format!("cant read {}", self.filename))?;

        let rule_file: RuleFile = serde_json::from_reader(f)?;
        self.result = LoadResult::Ok;
        self.content = Some(rule_file);

        // recursively include other rules
        for included_filename in self.included_filenames() {
            info!(
                "Recursively including [{}] - from [{}]",
                included_filename, self.filename
            );
            let included_file = Self::new(&included_filename, &self.include_path);
            match included_file.load(root_cache) {
                Ok(included_result) => {
                    info!("included result: {:?}", included_result);
                    self.include_set.insert(included_filename);
                }
                Err(x) => {
                    warn!("failed to include {:?}", x);
                }
            }
        }

        root_cache.insert(String::from(&self.filename), self);
        return Ok(LoadResult::Ok);
    }

    pub fn run_with_context(&self, state: &mut PicoState, context: &mut PicoContext) -> FnResult {
        match &self.content {
            Some(pico_rule) => pico_rule.run_with_context(state, context),
            None => Err(PicoError::Crash(String::from("no such rule"))),
        }
    }
}

pub enum LoaderError {
    NoSuchFile,
    ParseFailure,
}

/*
pub fn file_loader<T>( filename: &str ) -> Result<T, LoaderError>{

    let file =  File::open(filename).

}
*/

/*
pub fn load_file(
    filename: &str,
    loaded_filenames: &mut HashSet<RuleFileName>,
    loaded_rules: &mut LoadedRuleMap,
) -> Result<Rc<LoadedRuleFile>, PicoError> {
    info!("Loading... {}", filename);
    if loaded_filenames.contains(filename) {
        warn!("rulefile [{}] has already been included", filename);
        return Err(PicoError::ParseFailure {
            filename: String::from(filename),
        });
    }

    // errors with anyhow:error
    let f = File::open(filename).context(format!("CANT READ {}", filename))?;

    trace!("JSON parsing {}", filename);
    /*
        let jd = &mut serde_json::Deserializer::from_reader(&f);
        let r: Result<RuleFile, _> = serde_path_to_error::deserialize(jd);
        match r {
            Ok(_) => {}
            Err(err) => {
                let path = err.path().to_string();
                warn!(" PAAAATTTHHHH {}, {:?}", path, err);
            }
        }
    */

    let nf: RuleFile = serde_json::from_reader(f)?;

    // find all root level include directives
    let include_filenames: Vec<String> = nf
        .root
        .iter()
        .filter_map(|v| match v {
            RuleFileRoot::IncludeFile(f) => Some(f.include.filename.clone()),
            _ => None,
        })
        .collect();

    trace!(
        "loading included files {} / {:?}",
        filename,
        include_filenames
    );

    let mut lf: LoadedFile<RuleFile> = LoadedFile {
        result: LoadResult::Ok,
        content: Some(nf), // <- move nf into LoadedFile
        rulefile_cache: HashMap::new(),
    };

    trace!("collecting includes...");
    // load each lookup table into the cache
    //    let lookup_iter = nf.lookups.iter();
    //    for (key, table) in lookup_iter {
    //        lookup_cache.insert(key.to_string(), Rc::clone(table));
    //    }

    for ifilename in include_filenames {
        debug!("loading include file {} from {}", ifilename, filename);
        if ifilename != filename {
            match load_file(&ifilename, loaded_filenames, &mut lf.rulefile_cache) {
                Ok(included_rulefile) => {
                    for (including_filename, _included_file) in
                        included_rulefile.rulefile_cache.iter()
                    {
                        info!(
                            "loaded file [{}] also included: [{}]",
                            ifilename, including_filename
                        );
                    }
                }
                Err(e) => {
                    warn!("INSERT error {:?}", e);

                    let broken_file: LoadedFile<RuleFile> = LoadedFile {
                        result: LoadResult::NoSuchFile,
                        content: None,
                        rulefile_cache: HashMap::new(),
                    };
                    lf.rulefile_cache
                        .insert(String::from(ifilename), Rc::new(broken_file));
                }
            }
        }
    }

    let loading_file = Rc::new(lf);
    loaded_rules.insert(String::from(filename), Rc::clone(&loading_file));
    trace!("loading file {} FINISHED", filename);

    Ok(loading_file)
}
*/

/*
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
*/

pub type RuleFileName = String;

pub struct PicoRules {
    rulefile_names: HashSet<RuleFileName>, // all loaded rule filenames
    rulefile_cache: LoadedRuleMap,         // rc rulefiles
    entrypoint: String,                    // main entry rule filename
}

impl PicoRules {
    pub fn new(filename: &str) -> Self {
        PicoRules {
            rulefile_names: HashSet::new(),
            rulefile_cache: HashMap::new(),
            entrypoint: String::from(filename), // set the initial entrypoint filename
        }
    }

    pub fn load(&mut self) -> FnResult {
        trace!("Loading file");

        /*
        let lr = load_file(
            &self.entrypoint,
            &mut self.rulefile_names,
            &mut self.rulefile_cache,
        );

        */

        let mut lr = LoadedRuleFile::new(&self.entrypoint, &[].to_vec());

        trace!("LR : {:?}", lr);

        match lr.load(&mut self.rulefile_cache) {
            Ok(k) => {
                //trace!("populating lookups");
                //self.lookup_cache = populate_lookups(&mut self.rule_cache);
                trace!("LR2 : {:?}", self.rulefile_cache);

                for (k, v) in self.rulefile_cache.iter() {
                    trace!("KEY {} includes: {:?}", k, v.include_set);
                }

                Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
            }
            Err(e) => {
                error!("BAD thing: {}", e);
                Err(PicoError::AnyError(e))
            }
        }
    }

    pub fn make_state(&self) -> PicoState {
        let ps = PicoState::new(&self.rulefile_cache);

        debug!("MKAESTATE {:?}", self.rulefile_cache);
        return ps;
    }

    pub fn build(&mut self) -> &Self {
        self
    }

    pub fn run_with_context(&self, state: &mut PicoState, context: &mut PicoContext) {
        info!("entering at {}", self.entrypoint);
        trace!("RC {:?}", self.rulefile_cache);
        let loaded_file = self.rulefile_cache.get(&self.entrypoint).unwrap();

        /*
        loaded_file
            .content
            .as_ref()
            .unwrap()
            .run_with_context(state, context)
            .expect("something");
        */

        loaded_file.run_with_context(state, context);
        return;
    }
}
