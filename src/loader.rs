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

pub trait MyState<T = PicoRules> {
    fn set(self, filetype: T) -> Self;

    fn get(&self, filename: &str) -> &T;
}

pub struct PicoStateNew<'a, T = PicoRules> {
    rulefile_cache: &'a HashMap<String, T>,
}
impl<'a> PicoStateNew<'a> {
    fn new(rulefile_cache: &'a HashMap<String, PicoRules>) -> Self {
        PicoStateNew { rulefile_cache }
    }
}

impl MyState<PicoRules> for PicoStateNew<'_> {
    fn set(self, filetype: PicoRules) -> Self {
        self
    }

    fn get(&self, filename: &str) -> &PicoRules {
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

    pub fn load_includes(mut self) -> Self {
        for filename in &self.included_filenames() {
            let pr = PicoRules::new().load_rulefile(&filename).load_includes();

            self.rulefile_cache.insert(filename.to_string(), pr);
        }

        self
    }

    /*
     * load all included but unloaded files into the cache
     */

    pub fn make_state(&self) -> PicoStateNew {
        let ps = PicoStateNew::new(&self.rulefile_cache);
        ps
    }

    pub fn run_with_context(&self, state: &mut PicoStateNew, ctx: &mut PicoContext) {
        match &self.rulefile.content {
            Some(rule_file) => {
                rule_file.run_with_context_new(state, ctx);
            }
            None => {
                trace!("Cache-miss");
            }
        }
    }
}
