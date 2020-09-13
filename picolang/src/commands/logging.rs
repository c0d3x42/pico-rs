use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::commands::execution::{ActionExecution, ActionResult, ActionValue};
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
impl ActionExecution for Log {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> ActionResult {
        info!("MSG: {:?}", self.log);

        Ok(ActionValue::Continue)
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
impl ActionExecution for DebugLog {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ActionResult {
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

        let variables = ctx.local_variables.clone();

        //let k: HashMap<String, PicoValue> = l.into_iter().chain(c).collect();

        debug!("Rendering [{}]", self.tt);
        trace!(" with - {:?}", variables);
        let rendered = tt.render("debug", &variables);

        match rendered {
            Ok(val) => debug!("tmpl[{:?}]: {:?}", self.tt, val),
            Err(e) => error!("{:?}", e),
        }

        Ok(ActionValue::Continue)
    }
}
