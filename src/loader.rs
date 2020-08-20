use crate::commands::execution::Execution;
use crate::context::PicoContext;
use crate::rules::{RuleFile, RuleFileRoot};
use crate::state::PicoState;
use std::collections::HashMap;
use std::fs::File;

#[derive(Debug)]
enum FileStatus {
    Unchecked,
    Loaded,
    Missing,
}

#[derive(Debug)]
pub struct LoadedFile {
    content: Option<RuleFile>,
    status: FileStatus,
}

impl LoadedFile {
    pub fn new() -> Self {
        Self {
            content: None,
            status: FileStatus::Unchecked,
        }
    }

    pub fn load(mut self, rule_file: RuleFile) -> Self {
        self.content = Some(rule_file);
        self.status = FileStatus::Loaded;
        self
    }
}

pub trait MyState<T = LoadedFile> {
    fn set(self, filetype: T) -> Self;

    fn get(&self, filename: &str) -> &T;
}

pub struct PicoStateNew<'a, T = LoadedFile> {
    rulefile_cache: &'a HashMap<String, T>,
}
impl<'a> PicoStateNew<'a> {
    fn new(rulefile_cache: &'a HashMap<String, LoadedFile>) -> Self {
        PicoStateNew { rulefile_cache }
    }
}

impl MyState<LoadedFile> for PicoStateNew<'_> {
    fn set(self, filetype: LoadedFile) -> Self {
        self
    }

    fn get(&self, filename: &str) -> &LoadedFile {
        self.rulefile_cache.get(filename).unwrap()
    }
}

#[derive(Debug)]
pub struct PicoRules {
    rulefile_cache: HashMap<String, PicoRules>,
    entrypoint: String,
    rulefile: LoadedFile,
}

impl Default for PicoRules {
    fn default() -> Self {
        Self {
            rulefile_cache: HashMap::new(),
            entrypoint: String::new(),
            rulefile: LoadedFile::new(),
        }
    }
}

impl PicoRules {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_entry(mut self, entrypoint: &str) -> Self {
        self.entrypoint = entrypoint.to_string();
        self
    }

    pub fn load_rulefile(mut self, rulefile_name: &str) -> Self {
        match File::open(&rulefile_name) {
            Ok(opened_file) => {
                let rule_file: RuleFile = serde_json::from_reader(opened_file).unwrap();
                self.rulefile.content = Some(rule_file);
                self.rulefile.status = FileStatus::Loaded;
            }
            Err(x) => {
                error!("failed to open: {:?}", x);
                self.rulefile.status = FileStatus::Missing;
            }
        }
        self.set_entry(rulefile_name)
    }

    fn included_filenames(&self) -> Vec<String> {
        match &self.rulefile.content {
            Some(rf) => rf
                .root
                .iter()
                .filter_map(|r| match r {
                    RuleFileRoot::IncludeFile(f) => Some(f.include.filename.clone()),
                    _ => None,
                })
                .collect(),
            None => Vec::new(),
        }
    }

    /*
     * add a filename to the cache
     * not read in
    fn add_file(mut self, filename: &str) -> Self {
        let loaded_file = LoadedFile::new();
        self.rulefile_cache
            .insert(filename.to_string(), loaded_file);
        self
    }
     */

    /*
     * load all included but unloaded files into the cache
     */
    pub fn load(mut self) -> Self {
        for (filename, pico_rule) in self.rulefile_cache.iter_mut() {
            if let FileStatus::Loaded = pico_rule.rulefile.status {
                continue;
            }

            match File::open(&filename) {
                Ok(opened_file) => {
                    let rule_file: RuleFile = serde_json::from_reader(opened_file).unwrap();
                    loaded_file.content = Some(rule_file);
                    loaded_file.status = FileStatus::Loaded;
                }
                Err(x) => {
                    loaded_file.status = FileStatus::Missing;
                }
            }
        }
        self
    }

    pub fn make_state(&self) -> PicoStateNew {
        let ps = PicoStateNew::new(&self.rulefile_cache);
        ps
    }

    pub fn run_with_context(&self, state: &mut PicoStateNew, ctx: &mut PicoContext) {
        if let cache_entry = self.rulefile_cache.get(&self.entrypoint) {
            match cache_entry {
                Some(loaded_file) => match &loaded_file.content {
                    Some(rule_file) => {
                        rule_file.run_with_context_new(state, ctx);
                    }
                    None => {
                        trace!("Cache-miss");
                    }
                },
                None => {}
            }
        }
    }
}
