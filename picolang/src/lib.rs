#![warn(clippy::all)]
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate tinytemplate;

#[macro_use]
extern crate serde_derive;

pub use crate::values::{PicoValue, ValueProducer, Var, pico_value_as_truthy};

pub mod commands;
pub mod conditions;
pub mod context;
pub mod errors;
pub mod rules;
pub mod runtime;
pub mod types;
pub mod values;
