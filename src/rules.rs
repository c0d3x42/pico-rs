use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::commands::Command;
use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::include::IncludeFile;
use crate::lookups::Lookups;
//use crate::state::PicoState;
use crate::runtime::PicoRuntime;
use crate::values::PicoValue;

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludeFile {
    pub include: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RuleFileRoot {
    Command(Command),
    IncludeFile(IncludeFile),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleFile {
    pub root: Vec<RuleFileRoot>,
    #[serde(default = "RuleFile::default_version")]
    version: String,

    #[serde(default)]
    pub lookups: Lookups,
}

impl RuleFile {
    pub fn default_version() -> String {
        String::from("1.1")
    }
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

#[derive(Debug)]
enum FileStatus {
    Unchecked,
    Loaded,
    Missing,
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
        info!("Loading... {}", rulefile_name);
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
                    RuleFileRoot::IncludeFile(f) => Some(f.include.clone()),
                    _ => None,
                })
                .collect(),
            None => Vec::new(),
        }
    }

    /*
     * load all included but unloaded files into the cache
     */
    pub fn load_includes(mut self) -> Self {
        for filename in &self.included_filenames() {
            info!("including... {}", filename);
            let pr = PicoRules::new().load_rulefile(&filename).load_includes();

            self.rulefile_cache.insert(filename.to_string(), pr);
        }

        self
    }

    pub fn run_with_context(&self, runtime: &mut PicoRuntime, ctx: &mut PicoContext) {
        trace!("RUNTIME: {:?}", runtime.variables);

        runtime.add();
        runtime.set("key", "value");
        match &self.rulefile.content {
            Some(rule_file) => {
                for command in &rule_file.root {
                    match command {
                        RuleFileRoot::IncludeFile(i) => {
                            trace!("command include {:?}", i);
                            let pico_rule = self.rulefile_cache.get(&i.include).unwrap();

                            pico_rule.run_with_context(runtime, ctx);
                        }
                        RuleFileRoot::Command(c) => match c.run_with_context(&self, runtime, ctx) {
                            _ => {}
                        },
                    }
                }
                //rule_file.run_with_context_new(state, ctx);
            }
            None => {
                trace!("Cache-miss");
            }
        };
        runtime.remove();
    }
}
