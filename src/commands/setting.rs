use serde::{Deserialize, Serialize};

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::context::PicoContext;
//use crate::state::PicoState;
use crate::loader::PicoRules;
use crate::loader::PicoRuntime as PicoState;
use crate::values::{Extract, PicoValue, ValueProducer};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Settable {
    ValueProducing(String, ValueProducer),
    Extractor(Extract),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetCommand {
    set: Settable,
}
impl Execution for SetCommand {
    fn name(&self) -> String {
        "Set Command".to_string()
    }
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        state: &mut PicoState,
        ctx: &mut PicoContext,
    ) -> FnResult {
        info!("RUNNING SET");

        match &self.set {
            Settable::Extractor(extraction) => {
                let extracted_values = extraction.run_with_context(pico_rules, state, ctx)?;
                match extracted_values {
                    ExecutionResult::Setting(dict) => {
                        for (key, value) in dict {
                            ctx.set_value(&key, value);
                        }
                    }
                    _ => {}
                }
            }

            Settable::ValueProducing(var_name, value_producer) => {
                let produced_value = value_producer.run_with_context(pico_rules, state, ctx)?;

                debug!("Produced value = {:?}", produced_value);

                match produced_value {
                    ExecutionResult::Continue(pv) => match pv {
                        PicoValue::String(v) => {
                            ctx.set_value(var_name, PicoValue::String(v.to_string()))
                        }
                        PicoValue::Number(val) => ctx.set_value(var_name, PicoValue::Number(val)),
                        PicoValue::UnsignedNumber(val) => {
                            ctx.set_value(var_name, PicoValue::UnsignedNumber(val))
                        }
                        PicoValue::Boolean(val) => ctx.set_value(var_name, PicoValue::Boolean(val)),
                    },
                    ExecutionResult::Setting(dict) => {
                        // convert dictionary to ctx values prefixed by var_name

                        for (key, value) in dict {
                            let new_key = format!("{}{}", var_name, key);
                            ctx.set_value(&new_key, value);
                        }
                    }
                    everything_else => {
                        info!("produced value ignored {:?}", everything_else);
                    }
                }
            }
        }

        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}
