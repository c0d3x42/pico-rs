#![warn(clippy::all)]
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate serde;
extern crate serde_json;
extern crate tinytemplate;
extern crate valico;

#[macro_use]
extern crate serde_derive;

pub use crate::values::{PicoValue, ValueProducer, Var};

//pub use crate::include::IncludeFile;

pub mod app;
pub mod commands;
pub mod conditions;
pub mod context;
pub mod errors;
//pub mod include;
pub mod invar;
pub mod rules;
//pub mod runners;
//pub mod state;
pub mod runtime;
pub mod values;

pub mod nats;
pub mod server;
