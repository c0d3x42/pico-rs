
use serde_json::Value;
use picolang::types::Context;
use picolang::PicoValue;
use picolang::types::PicoRule;
use picolang::types::der::RuleFile;
use std::collections::HashMap;
use std::convert::TryFrom;


pub fn setup(rule: Value ) -> PicoRule {

  let rule_file: RuleFile = serde_json::from_value(rule).unwrap();
  //let data =  Context::new(ctx, globals);
  let pr = PicoRule::try_from(rule_file).unwrap();

  pr
}