use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::fs::File;

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::context::PicoContext;
use crate::errors::{PicoError, RuleFileError};
use crate::rules::RuleFile;
use crate::rules::RuleFileRoot;
//use crate::state::PicoState;
use crate::loader::PicoRuntime as PicoState;
use crate::values::PicoValue;

use valico::json_schema;

#[derive(Debug)]
pub struct IncludeFileDriver {
    pub filename: String,
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
    pub include: IncludeFileDriver,
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
                state.put_include_path(&self.include.filename);
                let include_result = rf.run_with_context(state, ctx);
                state.pop_include_path();
                trace!(
                    "result from included [{}] = {:?}",
                    &self.include.filename,
                    include_result
                );
                include_result
            }
            None => {
                warn!("unusable file XXX");

                Err(PicoError::UnusableFile {
                    filename: String::from(&self.include.filename),
                })
            }
        }
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
    pub content: Option<RuleFile>,
    include_set: HashSet<String>,
}

pub enum RequireModule {
    File(FileRequire),
    Null(NullRequire),
}

pub struct Require {
    url: String,
    r: RequireModule,
}

impl Require {
    pub fn new(include_reference: &str) -> Require {
        trace!("Require [{}]", include_reference);
        if include_reference.starts_with("file://") {
            let slice = &include_reference[7..]; // FIXME bounds checking
            trace!("slice [{}]", slice);

            Self {
                url: include_reference.to_string(),
                r: RequireModule::File(FileRequire::new(slice)),
            }
        } else {
            Self {
                url: include_reference.to_string(),
                r: RequireModule::Null(NullRequire::new(&include_reference)),
            }
        }
    }

    fn require(&self) -> Result<RuleFile, RuleFileError> {
        match &self.r {
            RequireModule::File(f) => f.require(),
            RequireModule::Null(n) => n.require(),
        }
    }
}

pub trait Requireer {
    fn require(&self) -> Result<RuleFile, RuleFileError>;
}

pub struct NullRequire {
    url: String,
}

impl NullRequire {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

impl Requireer for NullRequire {
    fn require(&self) -> Result<RuleFile, RuleFileError> {
        Err(RuleFileError::Unsuported {
            url: self.url.to_string(),
        })
    }
}

pub struct FileRequire {
    filename: String,
}

impl FileRequire {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }
}

impl Requireer for FileRequire {
    fn require(&self) -> Result<RuleFile, RuleFileError> {
        let opened_file =
            File::open(&self.filename).map_err(|source| RuleFileError::ReadError {
                source,
                filename: self.filename.to_string(),
            })?;

        let rule_file: RuleFile =
            serde_json::from_reader(opened_file).map_err(|source| RuleFileError::ParseError {
                source,
                filename: self.filename.to_string(),
            })?;

        Ok(rule_file)
    }
}

impl LoadedRuleFile {
    pub fn new(filename: &str, parent_path: &[String]) -> Self {
        let mut include_path: Vec<String> = parent_path.iter().map(|s| String::from(s)).collect();
        include_path.push(String::from(filename));
        trace!("LoadedRuleFile::new [{:?}]", include_path);

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
                included_filenames
            }
        }
    }

    pub fn require2(&mut self, requireer: Require) -> Result<&Self, RuleFileError> {
        let rule_file = requireer.require()?;
        self.result = LoadResult::Ok;
        self.content = Some(rule_file);
        Ok(self)
    }

    pub fn require(&mut self) -> Result<&Self, RuleFileError> {
        let opened_file =
            File::open(&self.filename).map_err(|source| RuleFileError::ReadError {
                source,
                filename: String::from(&self.filename),
            })?;
        let rule_file: RuleFile =
            serde_json::from_reader(opened_file).map_err(|source| RuleFileError::ParseError {
                source,
                filename: String::from(&self.filename),
            })?;
        self.result = LoadResult::Ok;
        self.content = Some(rule_file);

        Ok(self)
    }

    pub fn loader(mut self, root_cache: &mut LoadedRuleMap) {
        match self.require2(Require::new(&self.filename)) {
            Ok(loaded) => {
                let files_to_require: Vec<LoadedRuleFile> = loaded
                    .included_filenames()
                    .iter()
                    .map(|s| Self::new(s, &loaded.include_path))
                    .collect();

                for file in files_to_require {
                    file.loader(root_cache);
                }
            }
            Err(e) => {
                self.result = LoadResult::UnknownError;
                error!("{}", e);
                warn!("loader failed {:?}", self);
            }
        }
        root_cache.insert(String::from(&self.filename), self);
    }

    pub fn run_with_context(&self, state: &mut PicoState, context: &mut PicoContext) -> FnResult {
        match self.result {
            LoadResult::Ok => {}
            _ => {
                return Err(PicoError::UnusableFile {
                    filename: String::from(&self.filename),
                })
            }
        }

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
            entrypoint: filename.to_string(), // set the initial entrypoint filename
        }
    }

    pub fn load_file(mut self, filename: &str) -> Self {
        self
    }

    pub fn load(&mut self) -> FnResult {
        trace!("Loading file");

        let lr = LoadedRuleFile::new(&self.entrypoint, &[].to_vec());

        trace!("LR : {:?}", lr);

        lr.loader(&mut self.rulefile_cache);
        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }

    pub fn make_state(&self) -> PicoState {
        let ps = PicoState::new(&self.rulefile_cache, &self.entrypoint);

        //debug!("MKAESTATE {:?}", self.rulefile_cache);

        for (key, value) in &self.rulefile_cache {
            debug!("RULEFILE: [{}] = {:?}", key, value);
        }

        ps
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
