use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
use crate::values::PicoValue;

//use std::result;
use tinytemplate::TinyTemplate;

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    log: String,
}
impl Execution for Log {
    fn name(&self) -> String {
        "log".to_string()
    }
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &mut PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> FnResult {
        info!("MSG: {:?}", self.log);

        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DebugLog {
    debug: String,

    #[serde(default = "DebugLog::default_tt", skip)]
    tt: String,
}
impl DebugLog {
    fn default_tt() -> String {
        "TTT".to_string()
    }
}
impl Execution for DebugLog {
    fn name(&self) -> String {
        "debug-log".to_string()
    }
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        let mut tt = TinyTemplate::new();
        trace!("Building tiny template");

        match tt.add_template("debug", &self.debug) {
            Err(e) => {
                error!("template failure: {:?}", e);
                return Err(PicoError::Crash("Template failure".to_string()));
            }
            Ok(_) => {}
        }

        // combine variables and local_variables into one hashmap for template rendering

        let c = ctx.variables.clone();
        let l = ctx.local_variables.clone();

        let k: HashMap<String, PicoValue> = l.into_iter().chain(c).collect();

        let rendered = tt.render("debug", &k);
        trace!("MSG: {:?}, variables: {:#?}", self.debug, &ctx.variables);

        match rendered {
            Ok(val) => debug!("tmpl[{:?}]: {:?}", self.tt, val),
            Err(e) => error!("{:?}", e),
        }

        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}
