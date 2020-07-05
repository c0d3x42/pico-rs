use crate::command::{Execution, IfThenElse, RuleFileRoot};
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
    instructions: &Vec<RuleFileRoot>,
    context: &mut Context,
) -> Result<EndReason, String> {
    context
        .local_variables
        .insert("rrr".to_string(), PicoValue::Boolean(true));

    for instruction in instructions {
        match instruction {
            RuleFileRoot::IfThenElse(ite) => {
                info!("--> {:?}", ite.name());
                let run_result = ite.run_with_context(state, context);
                match run_result {
                    Ok(_) => {}
                    Err(_bad_thing) => return Err(format!("bad thing: {}", _bad_thing)),
                }
                info!("<-- {:?}", ite.name());
            }
            RuleFileRoot::IncludeFile(inc) => {
                info!("Including... {:?}", inc.name());
                let include_result = inc.run_with_context(state, context);
                match include_result {
                    Ok(_) => {}
                    Err(_bad_thing) => return Err(format!("bad thing: {}", _bad_thing)),
                }
            }
        }
    }

    return Ok(EndReason::EndReached);
}
