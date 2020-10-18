use super::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn json_logic_run(program: &str, data: &str) -> Result<Value, String>{

  let rule_file : der::RuleFile = serde_json::from_str(program).unwrap();
  let data : Value = serde_json::from_str(data).unwrap();

  let globals : HashMap<String, PicoValue> = HashMap::new();
  
  let mut ctx: Context = Context::new(&data, &globals);

  let pico_rule = PicoRule::try_from(rule_file).unwrap();

  pico_rule.run(&mut ctx)

}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_null(){
    let result = json_logic_run(r#"{"root":[]}"#, r#"{}"# );
    assert_eq!(result, Ok(json!(PicoValue::Null)))
  }

  #[test]
  fn test_var(){
    let result = json_logic_run(r#"{"root":[{"var":["a"]}]}"#, r#"{}"# );
    assert_eq!(result, Ok(json!(PicoValue::Null)))
  }

  #[test]
  fn test_var_var(){
    let result = json_logic_run(r#"{"root":[{"var":["a"]},{"var":["b"]}]}"#, r#"{}"# );
    assert_eq!(result, Ok(json!([PicoValue::Null, PicoValue::Null])))
  }

  #[test]
  fn test_var_a_is_a(){
    let result = json_logic_run(r#"{"root":[{"var":["a"]}]}"#, r#"{"a":"is a"}"# );
    assert_eq!(result, Ok(json!("is a")))
  }
  #[test]
  fn test_var_ab_is_ab(){
    let result = json_logic_run(r#"{"root":[{"var":["a"]}, {"var": ["b"]} ]}"#, r#"{"a":"is a", "b": "is b"}"# );
    assert_eq!(result, Ok(json!(["is a", "is b"])))
  }

  #[test]
  fn test_missing_var(){
    let result = json_logic_run(r#"{"root":[{"var":["missing"]} ]}"#, r#"{}"# );
    assert_eq!(result, Ok(json!(["is a", "is b"])))
  }




}