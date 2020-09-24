use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

use itertools::Itertools;
use std::marker::PhantomData;

pub mod loaders;
pub mod lookups;

use crate::commands::execution::ActionExecution;
use crate::commands::{Command, FiniCommand};
use crate::context::PicoContext;
use crate::runtime::PicoRuntime;
use crate::values::PicoValue;
use loaders::{FileLoader, PicoRuleLoader};
use lookups::{get_external_lookup_names, LookupType, Lookups};

#[derive(Serialize, Deserialize, Debug)]
pub struct StringOrSeq(#[serde(deserialize_with = "string_or_seq_string")] Vec<String>);

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludeFile {
    pub include: String,
    // namespaces the included file can access
    pub with_namespaces: Option<StringOrSeq>,
}

fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> serde::de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
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

///
/// The internal reprsentation of a Pico rule file
#[derive(Serialize, Deserialize, Debug)]
pub struct RuleFile {
    #[serde(default = "RuleFile::default_version")]
    pub version: String,

    #[serde(default)]
    pub lookups: Lookups,

    // optional namespaces this file creates
    pub namespaces: Option<Vec<String>>,

    pub root: Vec<RuleFileRoot>,

    pub fini: Option<Vec<RuleFileFini>>,
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
enum FileStatus {
    Loaded,
    Missing,
}

#[derive(Debug)]
pub struct PicoRules {
    rulename: String,
    rulefile: Option<RuleFile>,
    status: FileStatus,

    allowed_namespaces: HashSet<String>,
}

impl fmt::Display for PicoRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.rulefile {
            Some(rf) => write!(
                f,
                "PicoRule: {}, namespaces: [{}], rule summary: [{}]",
                self.rulename,
                self.allowed_namespaces.iter().join(", "),
                rf
            ),
            None => write!(
                f,
                "PicoRule: {}, namespaces: [{}], rule summary: [NOT LOADED]",
                self.rulename,
                self.allowed_namespaces.iter().join(", "),
            ),
        }
    }
}

impl Default for PicoRules {
    fn default() -> Self {
        Self {
            rulename: String::new(),
            rulefile: None,
            status: FileStatus::Missing,
            allowed_namespaces: HashSet::new(),
        }
    }
}

impl PicoRules {
    pub fn new(rulename: &str) -> Self {
        Self {
            rulename: rulename.to_string(),
            ..Default::default()
        }
    }

    pub fn all_namespace(&self, collected: &mut Vec<String>) {
        //let collected_namespace: Vec<String> = Vec::new();

        if let Some(rf) = &self.rulefile {
            if let Some(ns_v) = &rf.namespaces {
                for ns in ns_v {
                    collected.push(ns.to_string());
                }
            }
        }
    }

    pub fn get_rulefile(&self) -> Option<&RuleFile> {
        match &self.rulefile {
            None => None,
            Some(rulefile) => Some(&rulefile),
        }
    }

    pub fn set_rulename(mut self, rulename: &str) -> Self {
        self.rulename = rulename.to_string();
        self
    }

    pub fn get_rulename(&self) -> &String {
        &self.rulename
    }

    pub fn external_lookups(&self) -> Vec<(&String, &String)> {
        match &self.rulefile {
            Some(rf) => get_external_lookup_names(&rf.lookups),
            None => Vec::new(),
        }
    }

    pub fn load_rulefile(mut self, loader: impl PicoRuleLoader) -> Self {
        let s = &loader.filename_is();

        match loader.load() {
            Ok(rf) => {
                get_external_lookup_names(&rf.lookups);

                self.rulefile = Some(rf);
                self.status = FileStatus::Loaded;
            }
            Err(x) => {
                error!("failed to load {}", x);
                self.rulefile = None;
                self.status = FileStatus::Missing;
            }
        }

        trace!("After loading file SELF is {:?}", self);
        self
    }

    pub fn install_rulefile(mut self, rulefile_name: &str, rulefile: RuleFile) -> Self {
        self.rulefile = Some(rulefile);
        self.status = FileStatus::Loaded;
        self
    }

    // convenience, returns vec of filenames this file also includes
    fn _included_filenames(&self) -> Vec<String> {
        match &self.rulefile {
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
        let include_sections: Vec<&IncludeFile> = match &self.rulefile {
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

    pub fn setup_rules(self) -> Self {
        if let Some(rf) = &self.rulefile {
            if let Some(_namespaces) = &rf.namespaces {}
        }

        self
    }

    pub fn load_into_cache(filename: &str, cache: &mut HashMap<String, PicoRules>) {
        let f = FileLoader::new(filename);
        let pr = PicoRules::new(filename).load_rulefile(f);
        for x in pr.include_sections().iter() {
            PicoRules::load_into_cache(&x.include, cache);
        }
        cache.insert(filename.to_string(), pr);
    }

    pub fn upload_into_cache(
        filename: &str,
        rulefile: RuleFile,
        cache: &mut HashMap<String, PicoRules>,
    ) {
        let pr = PicoRules::new(filename).install_rulefile(filename, rulefile);
        cache.insert(filename.to_string(), pr);
    }

    pub fn run_with_context(&self, runtime: &PicoRuntime, ctx: &mut PicoContext) {
        trace!("RUNTIME: {:?}", runtime);

        match &self.rulefile {
            Some(rule_file) => {
                for command in &rule_file.root {
                    match command {
                        RuleFileRoot::IncludeFile(i) => {
                            // ensure the local scope variables are cleared
                            ctx.local_clear();
                            trace!("command include {:?}", i);
                            /*
                            if let Some(pico_rule) = self.rulefile_cache.get(&i.include) {
                                pico_rule.run_with_context(runtime, ctx);
                            } else {
                                error!("Did not find expected rule {}", &i.include);
                                trace!(" have these instead {:?}", self.rulefile_cache);
                            }
                            */
                        }
                        RuleFileRoot::Command(c) => match c.run_with_context(&self, runtime, ctx) {
                            _ => debug!("root: command finished"),
                        },
                    }
                }

                if let Some(fini_secion) = &rule_file.fini {
                    for fini_command in fini_secion {
                        match fini_command {
                            RuleFileFini::FiniCommand(fc) => {
                                match fc.run_with_context(&self, runtime, ctx) {
                                    Ok(data) => info!("returned data {:?}", data),
                                    Err(e) => error!("fini failed {}", e),
                                }
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
    }

    pub fn is_ns_allowed(&self, requested_namespace: &str) -> bool {
        debug!("checking namespace access for [{}]", requested_namespace);
        trace!("Allowed namespaces {:?}", self.allowed_namespaces);

        self.allowed_namespaces.contains(requested_namespace)
    }

    pub fn get_table(&self, table: &str) -> Option<&LookupType> {
        match &self.rulefile {
            None => None,
            Some(rule_file) => rule_file.lookups.get(table),
        }
    }

    pub fn table_lookup_value(&self, table: &str, key: &str) -> Option<&PicoValue> {
        match &self.rulefile {
            None => None,
            Some(rf) => match rf.lookups.get(table) {
                None => None,
                Some(m) => match m {
                    LookupType::InternalTable(table) => Some(table.lookup(key)),
                    _ => None,
                },
            },
        }
    }
}
