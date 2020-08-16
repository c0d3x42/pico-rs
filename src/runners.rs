use crate::commands::execution::Execution;
use crate::commands::flow_control::IfThenElse;
use crate::context::PicoContext;
use crate::rules::RuleFile;
use crate::state::PicoState;
use crate::values::PicoValue;

#[derive(Debug)]
pub enum EndReason {
    ExitCalled,
    EndReached,
    Crash(String),
}

fn _run_instruction(_ite: IfThenElse, _context: PicoContext) -> EndReason {
    EndReason::Crash("Not implemented".to_string())
}

pub fn run(
    state: &mut PicoState,
    rule: &RuleFile,
    context: &mut PicoContext,
) -> Result<EndReason, String> {
    context
        .local_variables
        .insert("rrr".to_string(), PicoValue::Boolean(true));

    match rule.run_with_context(state, context) {
        Ok(result) => {
            info!("End result {:?}", result);
            Ok(EndReason::EndReached)
        }
        Err(e) => {
            warn!("Failed {:?}", e);
            Ok(EndReason::Crash(String::from("Failed")))
        }
    }

    //return Ok(EndReason::EndReached);
}
