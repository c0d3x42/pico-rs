use anyhow::Result;

use picolang::rules::loaders::{FileLoader, StringLoader};
use picolang::rules::PicoRules;
use picolang::runtime::PicoRuntime;
use picolang::values::PicoValue;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<()> {
  env_logger::init();
  info!("hello2");

  //let loader = FileLoader::new("pico-rules.json");
  let document = r#"
  {
    "root": [
      {
        "log": "hello world"
      }
    ]
  }
  "#;

  let pico_rules = PicoRules::new().load_rulefile(StringLoader::new("anon.json", &document));

  let mut rt = PicoRuntime::new(&pico_rules)
    .add_global("my-version", &PicoValue::String("0.0.4".to_string()))
    .initialise();

  info!("made runtime");

  Ok(())
}
