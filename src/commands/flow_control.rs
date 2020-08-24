use serde::{Deserialize, Serialize};

use crate::commands::action::Action;
use crate::commands::execution::{
    ActionExecution, ActionResult, ActionValue, ConditionExecution, ConditionResult,
};
use crate::conditions::Condition;
use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
use crate::values::PicoValue;

use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct StopCommand {
    stop: String,
}
impl ActionExecution for StopCommand {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &mut PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> ActionResult {
        debug!("stopping because {:?}", self.stop);
        Ok(ActionValue::Stop(Some(self.stop.clone())))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BreakToCommand {
    r#break: uuid::Uuid,
}
impl ActionExecution for BreakToCommand {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &mut PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> ActionResult {
        debug!("breaking to {:?}", self.r#break);
        Ok(ActionValue::BreakTo(self.r#break))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IfThenElse {
    r#if: Condition,
    r#then: Action,
    r#else: Option<Action>,

    #[serde(default = "IfThenElse::default_uuid")]
    uuid: uuid::Uuid,
}
impl IfThenElse {
    fn default_uuid() -> uuid::Uuid {
        trace!("assigning default uuid");
        Uuid::new_v4()
    }
}

impl ActionExecution for IfThenElse {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ActionResult {
        info!("running ITE -> {:?}", self.uuid);
        let if_result: bool = self.r#if.run_with_context(pico_rules, runtime, ctx)?;

        match if_result {
            true => self.then.run_with_context(pico_rules, runtime, ctx),
            false => match &self.r#else {
                None => Ok(ActionValue::Continue),
                Some(else_branch) => else_branch.run_with_context(pico_rules, runtime, ctx),
            },
        }
    }
}
