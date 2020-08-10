//extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;
extern crate tinytemplate;
extern crate valico;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

use std::collections::HashMap;

use anyhow::Result;
use clap::{App, Arg};

extern crate picolang;

use picolang::context::PicoContext;
use picolang::include::PicoRules;
use picolang::values::PicoValue;

trait Initializable {
  fn init(&self) -> bool {
    return true;
  }
}

fn main() -> Result<()> {
  env_logger::init();
  let matches = App::new("Pico Lang")
    .version("0.1")
    .arg(
      Arg::new("rules")
        .long("rules")
        .default_value("pico-rule.json")
        .value_name("FILE")
        .takes_value(true),
    )
    .get_matches();
  info!("Matches {:?}", matches);

  if let Some(ref file) = matches.value_of("rules") {
    info!("filename {}", file);
  }

  debug!("Hello, world! ");

  let mut ctx = PicoContext::new();
  ctx
    .variables
    .insert("x".to_string(), PicoValue::String("xxxx".to_string()));
  ctx
    .variables
    .insert("q".to_string(), PicoValue::String("QQQQ".to_string()));
  ctx.variables.insert("n".to_string(), PicoValue::Number(42));
  ctx
    .variables
    .insert("op".to_string(), PicoValue::String("OP".to_string()));

  let mut sth: HashMap<String, String> = HashMap::new();
  sth.insert(String::from("a"), String::from("A"));

  if let Some(ref file) = matches.value_of("rules") {
    let mut pr = PicoRules::new(file);
    let x = pr.load();

    match x {
      Ok(y) => {
        info!("GOT y ");
        let mut ps = pr.make_state();
        pr.run_with_context(&mut ps, &mut ctx);
      }
      Err(e) => {
        warn!("OOPS {}", e);
      }
    }

    println!("\n FINAL FINAL CTX {:?}", ctx.local_variables);
  }
  Ok(())
}
