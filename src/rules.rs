use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

use itertools::Itertools;
use std::fs::File;

use crate::commands::execution::ActionExecution;
use crate::commands::{Command, FiniCommand};
use crate::context::PicoContext;
use crate::lookups::Lookups;
use crate::runtime::PicoRuntime;

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludeFile {
    pub include: String,
    pub namespaces: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RuleFileRoot {
    Command(Command),
    IncludeFile(IncludeFile),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RuleFileFini {
    FiniCommand(FiniCommand),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleFile {
    #[serde(default = "RuleFile::default_version")]
    version: String,

    #[serde(default)]
    pub lookups: Lookups,

    // optional namespaces this file creates
    pub namespaces: Option<Vec<String>>,

    pub root: Vec<RuleFileRoot>,

    pub fini: Vec<RuleFileFini>,
}

impl RuleFile {
    pub fn default_version() -> String {
        String::from("1.1")
    }
}
impl fmt::Display for RuleFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.namespaces {
            Some(ns) => write!(
                f,
                "version={}, rule count={} namespaces={}",
                self.version,
                self.root.len(),
                ns.join(",")
            ),
            None => write!(
                f,
                "version={}, rule count={}",
                self.version,
                self.root.len()
            ),
        }
    }
}

#[derive(Debug)]
pub struct LoadedFile {
    content: Option<RuleFile>,
    status: FileStatus,
}

impl fmt::Display for LoadedFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.content {
            Some(c) => write!(f, "{:?}, {}", self.status, c),
            None => write!(f, "{:?}", self.status,),
        }
    }
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

    allowed_namespaces: HashSet<String>,
}

impl fmt::Display for PicoRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PicoRule: {}, includes: [{}], namespaces: [{}], rule summary: [{}]",
            self.entrypoint,
            self.rulefile_cache.keys().join(", "),
            self.allowed_namespaces.iter().join(", "),
            self.rulefile
        )
    }
}

impl Default for PicoRules {
    fn default() -> Self {
        Self {
            rulefile_cache: HashMap::new(),
            entrypoint: String::new(),
            rulefile: LoadedFile::new(),
            allowed_namespaces: HashSet::new(),
        }
    }
}

impl PicoRules {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn all_namespace(&self, collected: &mut Vec<String>) {
        //let collected_namespace: Vec<String> = Vec::new();

        if let Some(rf) = &self.rulefile.content {
            if let Some(ns_v) = &rf.namespaces {
                for ns in ns_v {
                    collected.push(ns.to_string());
                }
            }
        }
        for (_key, pico_rule) in &self.rulefile_cache {
            PicoRules::all_namespace(pico_rule, collected);
        }
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

    // convenience, returns vec of filenames this file also includes
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

    fn include_sections(&self) -> Vec<&IncludeFile> {
        let include_sections: Vec<&IncludeFile> = match &self.rulefile.content {
            Some(rfc) => rfc
                .root
                .iter()
                .filter_map(|r| match r {
                    RuleFileRoot::IncludeFile(f) => Some(f),
                    _ => None,
                })
                .collect(),
            None => Vec::new(),
        };

        include_sections
    }

    pub fn setup_rules(mut self) -> Self {
        if let Some(rf) = &self.rulefile.content {
            if let Some(namespaces) = &rf.namespaces {}
        }

        self
    }

    /*
     * load all included but unloaded files into the cache
     */
    pub fn load_includes(mut self) -> Self {
        let imported_rules: Vec<PicoRules> = self
            .include_sections()
            .iter()
            .map(|i| {
                info!("includes: [{}]", i.include);

                info!("permitted namespace [{:?}]", i.namespaces);

                let mut imported_pico_rule =
                    PicoRules::new().load_rulefile(&i.include).load_includes();

                if let Some(allowed_namespaces) = &i.namespaces {
                    for ns in allowed_namespaces {
                        imported_pico_rule.allowed_namespaces.insert(ns.to_string());
                    }
                }
                imported_pico_rule
            })
            .collect();

        for pr in imported_rules {
            info!("Importing {}", pr);
            self.rulefile_cache.insert(pr.entrypoint.to_string(), pr);
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
                for fini_command in &rule_file.fini {
                    match fini_command {
                        RuleFileFini::FiniCommand(fc) => {
                            match fc.run_with_context(&self, runtime, ctx) {
                                Ok(data) => info!("returned data {:?}", data),
                                Err(e) => {}
                            }
                        }
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

    pub fn is_ns_allowed(&self, requested_namespace: &str) -> bool {
        self.allowed_namespaces.contains(requested_namespace)
    }
}
