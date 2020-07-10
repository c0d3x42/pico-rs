//extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;
extern crate tinytemplate;
extern crate valico;
#[macro_use]
extern crate serde_derive;

use serde::{de::DeserializeSeed, Deserialize, Deserializer};
#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::fs::File;

extern crate picolang;

use picolang::command::RuleFile;
use picolang::context::{Context, PicoState};
use picolang::include::{load_file, LoadedCache};
use picolang::values::PicoValue;

trait Initializable {
  fn init(&self) -> bool {
    return true;
  }
}

/*
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
*/

fn main() {
  env_logger::init();

  /*
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
  */
  let pico_rule: RuleFile = serde_json::from_reader(File::open("pico-rule.json").unwrap()).unwrap();
  info!("Pico rules: {:?}", pico_rule);

  let mut ctx = Context::new();
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

  let empty_map = HashMap::new();

  let some_lookups = match &pico_rule.lookups {
    Some(l) => l,
    None => &empty_map,
  };

  let mut ps = PicoState::new(&some_lookups);

  info!("PS = {:?}", ps);

  //for _x in 0..100000 {
  let result = picolang::runners::run(&mut ps, &pico_rule, &mut ctx);
  match result {
    Ok(_) => info!("OK"),
    Err(e) => warn!("oopsie : {}", e),
  }
  //}

  let j = serde_json::to_string(&pico_rule.root);

  println!("JSON = {:?}", j);

  println!("PS = {:?}", ps);
  println!("Final CTX {:?}", ctx.local_variables);

  let mut cache: LoadedCache<RuleFile> = HashMap::new();
  load_file("other-rules.json", &mut cache);

  println!("cache: {:?}", cache);
}
