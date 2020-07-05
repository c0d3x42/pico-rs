use serde::{Deserialize, Deserializer, Serialize};
use serde_json;
use std::fs::File;

use crate::command::RuleFile;
use crate::command::{Execution, ExecutionResult, FnResult};
use crate::context::{Context, PicoState};
use crate::errors::PicoError;
use crate::values::PicoValue;

#[derive(Serialize, Debug)]
pub struct IncludeFileDriver(String);

impl<'de> Deserialize<'de> for IncludeFileDriver {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        debug!("deserializing.1 ");
        let s = String::deserialize(deserializer)?;
        debug!("deserializing {:?}", s);

        let nf: RuleFile = serde_json::from_reader(File::open(s).unwrap()).unwrap();
        debug!("NEW RULE FILE {:?}", nf);

        Ok(IncludeFileDriver(String::from("lop")))
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

    fn run_with_context(&self, _state: &mut PicoState, _ctx: &mut Context) -> FnResult {
        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}
