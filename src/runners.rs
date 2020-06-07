use crate::command::{Execution, ExecutionResult, IfThenElse};
use crate::context::pico::Context;

#[derive(Debug)]
pub enum EndReason {
    ExitCalled,
    EndReached,
    Crash(String),
}

fn run_instruction(ite: IfThenElse, context: Context) -> EndReason {
    EndReason::Crash(format!("Not implemented"))
}

pub fn run(instructions: Vec<IfThenElse>, context: Context) -> Result<EndReason, String> {
    for instruction in &instructions {
        info!("--> {:?}", instruction.name());
        let run_result = instruction.run_with_context(&context.variables);
        match run_result {
            Ok(_) => {}
            Err(_bad_thing) => return Err("bad thing".to_string()),
        }
        info!("<-- {:?}", instruction.name());
    }

    return Ok(EndReason::EndReached);
}
