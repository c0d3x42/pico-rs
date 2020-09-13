use serde::{Deserialize, Serialize};

use crate::commands::execution::{ActionExecution, ActionResult, ActionValue, ValueExecution};
use crate::context::PicoContext;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
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
    namespaces: Option<Vec<String>>, //namespaces that the variable will be available in
}
impl ActionExecution for SetCommand {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ActionResult {
        info!("RUNNING SET for {}", pico_rules);

        match &self.set {
            Settable::Extractor(extraction) => {
                let extracted_values = extraction.run_with_context(pico_rules, runtime, ctx)?;
                match extracted_values {
                    PicoValue::Object(dict) => {
                        for (key, value) in dict {
                            ctx.local_set(&key, &value);
                        }
                    }
                    _ => warn!("extractor returned non Object"),
                }
            }

            Settable::ValueProducing(var_name, value_producer) => {
                trace!(
                    "Setting from valueproducer [{}] with {:?}",
                    var_name,
                    value_producer
                );

                let produced_value = value_producer.run_with_context(pico_rules, runtime, ctx)?;

                debug!("Produced value = {:?}", produced_value);
                trace!("available namespaces: {:?}", self.namespaces);
                match &self.namespaces {
                    None => {}
                    Some(requested_namespaces) => {
                        for ns in requested_namespaces {
                            if pico_rules.is_ns_allowed(ns) {
                                ctx.ns_set(ns, var_name, &produced_value);
                            } else {
                                warn!("namespace {}, access denied for {}", ns, var_name);
                            }
                        }
                    }
                }
                ctx.local_set(var_name, &produced_value);
            }
        }
        trace!("CTX now {:?}", ctx);

        Ok(ActionValue::Continue)
    }
}
