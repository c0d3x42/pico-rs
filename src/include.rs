use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use std::fs::File;

use crate::command::RuleFile;
use crate::command::{Execution, FnResult};
use crate::context::{Context, PicoState};

#[derive(Debug)]
pub struct IncludeFileDriver {
    _filename: String,
    rule: RuleFile,
}

impl<'de> Deserialize<'de> for IncludeFileDriver {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // use serde::de::Error;
        debug!("deserializing.1 ");
        let s = String::deserialize(deserializer)?;
        debug!("deserializing {:?}", s);

        let nf: RuleFile = serde_json::from_reader(File::open(&s).unwrap()).unwrap();
        debug!("NEW RULE FILE {:?}", nf);

        Ok(IncludeFileDriver {
            _filename: s,
            rule: nf,
        })
    }
}

impl Serialize for IncludeFileDriver {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // FIXME: needs to write out the self.rule to self._filename
        serializer.serialize_str(&self._filename)
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

    fn run_with_context(&self, state: &mut PicoState, ctx: &mut Context) -> FnResult {
        info!("running included module");
        self.include.rule.run_with_context(state, ctx)
        //Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}
