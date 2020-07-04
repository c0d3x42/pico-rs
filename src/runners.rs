use crate::command::{Execution, IfThenElse};
use crate::context::{Context, PicoState};
use crate::values::PicoValue;

#[derive(Debug)]
pub enum EndReason {
    ExitCalled,
    EndReached,
    Crash(String),
}

fn _run_instruction(_ite: IfThenElse, _context: Context) -> EndReason {
    EndReason::Crash(format!("Not implemented"))
}

pub fn run(
    state: &mut PicoState,
    instructions: &Vec<IfThenElse>,
    context: &mut Context,
) -> Result<EndReason, String> {
    context
        .local_variables
        .insert("rrr".to_string(), PicoValue::Boolean(true));

    for instruction in instructions {
        info!("--> {:?}", instruction.name());
        let run_result = instruction.run_with_context(state, context);
        match run_result {
            Ok(_) => {}
            Err(_bad_thing) => return Err(format!("bad thing: {}", _bad_thing)),
        }
        info!("<-- {:?}", instruction.name());
    }

    return Ok(EndReason::EndReached);
}
