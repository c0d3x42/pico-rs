extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;
extern crate tinytemplate;
extern crate valico;

#[macro_use]
extern crate log;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use valico::json_schema;

mod command;
use crate::command::{IfThenElse, RuleFile};

mod runners;

mod context;
use crate::context::{pico::Context, pico::PicoContext, pico::PicoHashMap};

mod pathing;
use crate::pathing::{path::PathLookup, path::PathLookupAugmented};

mod variables;
use crate::variables::literal::VarLiteral;
use crate::variables::var::Var;

mod logic;
use crate::logic::branching::PicoIfThenElse;
use crate::logic::collections::PicoConditions;
use crate::logic::equality::PicoConditionEquality;

trait Initializable {
  fn init(&self) -> bool {
    return true;
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct VarLookup {
  var: VarLiteral,
}
impl Executable for VarLookup {
  fn exec(&self, hm: &PicoHashMap) -> bool {
    debug!("VarLookup");
    return self.var.exec(hm);
  }
}

trait Executable {
  fn exec(&self, _hm: &PicoHashMap) -> bool {
    return true;
  }
}

struct ContextVars {
  hm: HashMap<String, String>,
}
impl ContextVars {
  fn new() -> ContextVars {
    ContextVars { hm: HashMap::new() }
  }
}

fn main() {
  env_logger::init();

  info!("Starting up");

  debug!("Hello, world!");
  let json_v4_schema: Value =
    serde_json::from_reader(File::open("schema/schema.json").unwrap()).unwrap();

  debug!("schema is {:?}", json_v4_schema);
  let mut scope = json_schema::Scope::new();
  let schema = scope
    .compile_and_return(json_v4_schema.clone(), false)
    .unwrap();

  debug!("Is valid: {}", schema.validate(&json_v4_schema).is_valid());

  let json_rules: PicoIfThenElse =
    serde_json::from_reader(File::open("pico.json").unwrap()).unwrap();
  debug!("Pico rules: {:?}", json_rules);

  let mut oo = ContextVars::new();
  oo.hm.insert("bob".to_string(), "boooob".to_string());
  oo.hm.insert("lop".to_string(), "LOOOOB".to_string());
  let mut ctx = PicoContext::new();
  ctx.put("dddd", "llll");
  info!("CTX {:?}", ctx);

  let mut hm: HashMap<String, String> = HashMap::new();
  hm.insert("lop".to_ascii_lowercase(), "bingo".into());
  let value = hm.get("lop");
  if let Some(v) = value {
    debug!("FOUND {:?}", v);
  }

  let t = PicoContext::new();
  debug!("PC {:?}", t);
  let got = t.get("lop");
  if let Some(vvv) = got {
    debug!("VVVV: {:?}", vvv);
  }

  let truth = json_rules.exec(t.values);
  debug!("Truth: {:?}", truth);

  let a = VarLiteral::S("lll".to_string());
  let json_obj = json!({ "store": {}});
  let mut selector = jsonpath::selector(&json_obj);
  let t = selector("$.");

  debug!("finish");
  warn!("DONE");

  let pico_rule: RuleFile = serde_json::from_reader(File::open("pico-rule.json").unwrap()).unwrap();
  info!("Pico rules: {:?}", pico_rule);

  let mut ctx = Context::new();
  ctx.variables.insert(
    "x".to_string(),
    crate::command::Value::String("xxxx".to_string()),
  );
  ctx.variables.insert(
    "q".to_string(),
    crate::command::Value::String("QQQQ".to_string()),
  );

  let result = crate::runners::run(&pico_rule.root, ctx);
  match result {
    Ok(_) => info!("OK"),
    Err(e) => warn!("oopsie : {}", e),
  }

  let j = serde_json::to_string(&pico_rule.root);

  println!("JSON = {:?}", j);
}
