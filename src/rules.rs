use serde::{Deserialize, Serialize};

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::commands::Command;
use crate::context::PicoContext;
use crate::errors::PicoError;
use crate::include::IncludeFile;
use crate::loader::PicoStateNew;
use crate::lookups::Lookups;
use crate::state::PicoState;
use crate::values::PicoValue;

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

    pub fn run_with_context_new(
        &self,
        state: &mut PicoStateNew,
        ctx: &mut PicoContext,
    ) -> FnResult {
        for instruction in &self.root {
            match instruction {
                x => debug!("ROOT value {:?}", x),
            }
        }
        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}

impl Execution for RuleFile {
    fn name(&self) -> String {
        "rule-file".to_string()
    }

    fn run_with_context(&self, state: &mut PicoState, context: &mut PicoContext) -> FnResult {
        info!("Running rules from {}", state.get_include_path());

        for instruction in &self.root {
            match instruction {
                RuleFileRoot::Command(c) => match c.run_with_context(state, context) {
                    _ => {}
                },

                RuleFileRoot::IncludeFile(inc) => {
                    info!("Running Included... {:?}", inc.name());
                    let include_result = inc.run_with_context(state, context);
                    match include_result {
                        Ok(_) => {}
                        Err(_bad_thing) => {
                            return Err(PicoError::Crash(format!("bad thing: {}", _bad_thing)))
                        }
                    }
                }
            }
        }
        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}
