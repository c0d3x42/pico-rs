use crate::commands::execution::Execution;
use crate::context::PicoContext;
use crate::rules::{RuleFile, RuleFileRoot};
//use crate::state::PicoState;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;
use std::sync::Arc;

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
    runner: PicoFileRunner,
}
impl<'a> PicoStateNew<'a> {
    fn new(rulefile_cache: &'a HashMap<String, PicoRules>) -> Self {
        PicoStateNew {
            rulefile_cache,
            runner: PicoFileRunner::new(Parent::Root),
        }
    }

    pub fn get_var(&self, key: &str) -> Option<&String> {
        self.runner.get_local(key)
    }

    pub fn set_var(&mut self, key: &str, value: &str) {
        self.runner.set_local(key, value)
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

    pub fn make_state(&self) -> PicoStateNew {
        let ps = PicoStateNew::new(&self.rulefile_cache);
        ps
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
                            let filename = i.include.clone();
                            let pico_rule = self.rulefile_cache.get(&filename).unwrap();

                            pico_rule.run_with_context(runtime, ctx);
                            //let runner = PicoFileRunner::new(Parent::Parent(Rc::new(&runtime)));
                            //pico_rule.run_with_context(state, runner, ctx);
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

pub struct PicoRuntime<'a> {
    variables: Vec<HashMap<String, String>>,
    current_rules: Vec<&'a PicoRules>,
    root_rule: &'a PicoRules, // borrowed reference of the top level rulefile
}
impl<'a> PicoRuntime<'a> {
    pub fn new(root_rule: &'a PicoRules) -> Self {
        Self {
            variables: Vec::new(),
            current_rules: Vec::new(),
            root_rule,
        }
    }

    pub fn exec_root_with_context(&mut self, ctx: &mut PicoContext) {
        self.root_rule.run_with_context(self, ctx)
    }

    /*
     * Maybe...
     * when switching to an included file,
     *  push the include onto current_rules
     *   exec the last current_rules
     *  pop off current rules
     */
    pub fn exec_current_with_context(&mut self, ctx: &mut PicoContext) {
        self.current_rules
            .last_mut()
            .unwrap()
            .run_with_context(self, ctx);
    }

    pub fn add(&mut self) {
        info!("runtimes: {}", self.variables.len());
        self.variables.push(HashMap::new())
    }

    pub fn remove(&mut self) {
        self.variables.pop();
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        for variable in self.variables.iter() {
            match variable.get(key) {
                Some(v) => return Some(v),
                None => continue,
            }
        }
        None
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if let Some(mut hm) = self.variables.pop() {
            hm.insert(key.to_string(), value.to_string());
            self.variables.push(hm);
        }
    }
}

pub enum Parent {
    Root,
    Parent(Rc<PicoFileRunner>),
}

pub struct PicoFileRunner {
    locals: HashMap<String, String>,
    parent: Parent,
}

impl PicoFileRunner {
    pub fn new(parent: Parent) -> Self {
        Self {
            locals: HashMap::new(),
            parent,
        }
    }

    pub fn n2(c: &Rc<Self>) -> Self {
        Self {
            locals: HashMap::new(),
            parent: Parent::Parent(Rc::clone(c)),
        }
    }

    pub fn root() -> Rc<Self> {
        Rc::new(Self {
            locals: HashMap::new(),
            parent: Parent::Root,
        })
    }

    pub fn make_child(&self) -> Self {
        //PicoFileRunner::new(Parent::Parent(Rc::new(self)))
        PicoFileRunner::new(Parent::Root)
    }

    pub fn get_local(&self, key: &str) -> Option<&String> {
        match self.locals.get(key) {
            Some(v) => Some(v),
            None => match &self.parent {
                Parent::Root => None,
                Parent::Parent(p) => p.get_local(key),
            }, //        None => self.locals_parent.iter().find_map(|d| d.get(key)),
        }
    }

    pub fn set_local(&mut self, key: &str, value: &str) {
        self.locals.insert(key.to_string(), value.to_string());
    }
}
