use std::collections::HashMap;

use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
//use crate::values::PicoValue;
use anyhow::Result as AnyHowResult;
use serde_json::Value as PicoValue;

#[derive(Clone, Debug)]
pub enum ActionValue {
    Continue,
    Setting(HashMap<String, PicoValue>),
    Stop(Option<String>),
    BreakTo(uuid::Uuid),
}

pub type MyResult<T> = AnyHowResult<T, PicoError>;
pub type ConditionResult = MyResult<bool>;
pub type ValueResult = MyResult<PicoValue>;
pub type ActionResult = MyResult<ActionValue>;

// pub type FnResult = Result<ExecutionResult, PicoError>;

pub trait ConditionExecution {
    fn run_with_context(
        &self,
        pico_rule: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> MyResult<bool> {
        Err(PicoError::Crash("Not implemented".to_string()))
    }
}

pub trait ValueExecution {
    fn run_with_context(
        &self,
        pico_rule: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> MyResult<PicoValue> {
        Err(PicoError::Crash("Not implemented".to_string()))
    }
}

pub trait ActionExecution {
    fn run_with_context(
        &self,
        pico_rule: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> MyResult<ActionValue> {
        Err(PicoError::Crash("Not implemented".to_string()))
    }
}

#[test]
fn has_name() {
    assert_eq!(2 + 2, 4);
}
