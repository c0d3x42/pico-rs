#![warn(clippy::all)]
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate serde;
extern crate serde_json;
extern crate tinytemplate;

#[macro_use]
extern crate serde_derive;

pub use crate::values::{PicoValue, ValueProducer, Var};

pub mod commands;
pub mod conditions;
pub mod context;
pub mod errors;
pub mod rules;
pub mod runtime;
pub mod values;
