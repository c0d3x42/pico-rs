mod common;

use picolang::types::Context;
use picolang::PicoValue;
use serde_json;
use serde_json::json;
use std::collections::HashMap;
/*
#[test]
fn eq_string_string(){


  let rule = json!({ "version": "1.2",
    "root":[
    {"if": [ {"==": ["test1", "test1"]}]}
  ]});

  let pr = common::setup(rule );
  let globals: HashMap<String, PicoValue> = HashMap::new();
  let data = json!({});
  let mut ctx = Context::new(&data, &globals);
  let result = pr.exec( &mut ctx);

  assert!(result.is_ok());

  assert_eq!(result.unwrap(), PicoValue::Bool(true))
}
*/
