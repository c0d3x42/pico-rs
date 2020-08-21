use std::collections::HashMap;

use crate::context::PicoContext;
use crate::errors::PicoError;
//use crate::state::PicoState;
use crate::loader::PicoRules;
use crate::loader::PicoRuntime as PicoState;
use crate::values::PicoValue;
use anyhow::Result as AnyHowResult;

#[derive(Clone, Debug)]
pub enum ExecutionResult {
    Continue(PicoValue),
    Setting(HashMap<String, PicoValue>),
    Stop(Option<String>),
    BreakTo(uuid::Uuid),
}

pub type MyResult<T> = AnyHowResult<T, PicoError>;
pub type FnResult = MyResult<ExecutionResult>;

// pub type FnResult = Result<ExecutionResult, PicoError>;

pub trait Execution {
    fn name(&self) -> String;
    fn alises(&self) -> Vec<String> {
        vec![]
    }
    fn run(&self) -> FnResult {
        Err(PicoError::Crash("Not implemented".to_string()))
        /*
        Err(ErrorResult::Crash(
            format!("Not done for: {}", &self.name()).to_string(),
        ))
        */
    }

    fn run_with_context(
        &self,
        pico_rule: &PicoRules,
        state: &mut PicoState,
        ctx: &mut PicoContext,
    ) -> FnResult {
        Err(PicoError::Crash("Not implemented".to_string()))
    }

    fn Xrun_with_context(&self, _state: &mut PicoState, _ctx: &mut PicoContext) -> FnResult {
        trace!("Running with context for: {}", &self.name());
        Err(PicoError::Crash("Not implemented".to_string()))
    }
}

#[test]
fn has_name() {
    assert_eq!(2 + 2, 4);
}
