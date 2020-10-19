use super::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn json_logic_run(program: &str, data: &str) -> Result<Value, String> {
  //  env_logger::init();

  let rule_file: der::RuleFile = serde_json::from_str(program).unwrap();
  let data: Value = serde_json::from_str(data).unwrap();

  let globals: HashMap<String, PicoValue> = HashMap::new();

  let mut ctx: Context = Context::new(&data, &globals);

  let pico_rule = PicoRule::try_from(rule_file).map_err(|e| format!("{}", e))?;

  info!("running rule");
  pico_rule.run(&mut ctx)
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn test_null() {
    let result = json_logic_run(r#"{"root":[]}"#, r#"{}"#);
    debug!("test_null {:?}", result);
    assert!(result.is_ok());
    assert_eq!(result, Ok(json!(PicoValue::Null)))
  }

  #[test]
  fn test_var() {
    let result = json_logic_run(r#"{"root":[{"var":["a"]}]}"#, r#"{}"#);
    assert_eq!(result, Ok(json!(PicoValue::Null)))
  }

  #[test]
  fn test_var_var() {
    let result = json_logic_run(r#"{"root":[{"var":["a"]},{"var":["b"]}]}"#, r#"{}"#);
    assert_eq!(result, Ok(json!([PicoValue::Null, PicoValue::Null])))
  }

  #[test]
  fn test_var_a_is_a() {
    let result = json_logic_run(r#"{"root":[{"var":["a"]}]}"#, r#"{"a":"is a"}"#);
    assert!(result.is_ok());
    assert_eq!(result, Ok(json!("is a")))
  }
  #[test]
  fn test_var_ab_is_ab() {
    let result = json_logic_run(
      r#"{"root":[{"var":["a"]}, {"var": ["b"]} ]}"#,
      r#"{"a":"is a", "b": "is b"}"#,
    );
    assert_eq!(result, Ok(json!(["is a", "is b"])))
  }

  mod equality {
    use super::*;

    #[test]
    fn test_has_two_exprs() {
      let result = json_logic_run(
        r#"{"root": [{"==": [{"var": "a"}, {"var":"b"} ] }] }"#,
        r#"{}"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!(Value::Bool(true))))
    }

    #[test]
    fn test_two_strings_are_equal() {
      let result = json_logic_run(
        r#"{"root": [{"==": [{"var": "a"}, {"var":"b"} ] }] }"#,
        r#"{"a": "matches", "b": "matches"}"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!(Value::Bool(true))))
    }
  }

  mod expr_path {

    use super::*;

    #[test]
    fn jmes_one() {
      init();
      let result = json_logic_run(
        r#"{"root": [{"var": "a", "type": "path"} ] }"#,
        r#"{"a": "matches", "b": "matches"}"#,
      );

      assert_eq!(result, Ok(json!("matches")))
    }

    #[test]
    fn jmes_two() {
      init();
      let result = json_logic_run(
        r#"{"root": [{"var": "a", "type": "path"} ] }"#,
        r#"{"a": { "b": "matchesab"} }"#,
      );

      assert_eq!(result, Ok(json!({"b": "matchesab"})))
    }
  }

  mod expr_and {
    use super::*;

    #[test]
    fn test_one_and() {
      let result = json_logic_run(r#"{"root": [{"and": [{"var": "a"} ] }] }"#, r#"{}"#);
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!(Value::Null)))
    }

    #[test]
    fn test_two_and_first() {
      let result = json_logic_run(
        r#"{"root": [{"and": [{"var": "a"}, {"var": "b"} ] }] }"#,
        r#"{"b": "b" }"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(Value::Null))
    }

    #[test]
    fn test_two_and_last() {
      let result = json_logic_run(
        r#"{"root": [{"and": [{"var": "a"}, {"var": "b"} ] }] }"#,
        r#"{"a": "a", "b": "b" }"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!("b")))
    }
  }

  mod expr_or {
    use super::*;

    #[test]
    fn test_one_or() {
      let result = json_logic_run(r#"{"root": [{"or": [{"var": "a"} ] }] }"#, r#"{}"#);
      assert!(result.is_err());
      assert_eq!(result, Err("Invalid PicoRule".to_string()))
    }

    #[test]
    fn test_two_or() {
      let result = json_logic_run(
        r#"{"root": [{"or": [{"var": "a"}, {"var": "b"} ] }] }"#,
        r#"{"b": "b" }"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!("b")))
    }
  }

  mod expr_if {
    use super::*;

    #[test]
    fn test_if_no_then() {
      let result = json_logic_run(
        r#"{"root": [{"if": [{"var": "a"} ] }] }"#,
        r#"{"a": "a", "b":"b"}"#,
      );
      assert_eq!(result, Err("Invalid PicoRule".to_string()))
    }

    #[test]
    fn test_if_then_return_string() {
      let result = json_logic_run(
        r#"{"root": [{"if": [{"var": "a"}, {"var":"b"}] }] }"#,
        r#"{"a": "a", "b":"b"}"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!("b")))
    }

    #[test]
    fn test_if_then_return_number() {
      let result = json_logic_run(
        r#"{"root": [{"if": [{"var": "a"}, {"var":"b"}] }] }"#,
        r#"{"a": "a", "b":123}"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!(123)))
    }

    #[test]
    fn test_if_then_return_true() {
      let result = json_logic_run(
        r#"{"root": [{"if": [{"var": "a"}, {"var":"b"}] }] }"#,
        r#"{"a": "a", "b":true}"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!(Value::Bool(true))))
    }

    #[test]
    fn test_if_then_return_null() {
      let result = json_logic_run(
        r#"{"root": [{"if": [{"var": "a"}, {"var":"b"}] }] }"#,
        r#"{"a": "a", "b":null}"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!(Value::Null)))
    }
  }

  mod expr_lt {
    use super::*;

    #[test]
    fn test_lt_one() {
      let result = json_logic_run(r#"{"root": [{"<": [{"var": "a"} ] }] }"#, r#"{"a": 1 }"#);
      assert!(result.is_err());
      assert_eq!(result, Err("Invalid PicoRule".to_string()))
    }

    #[test]
    fn test_lt_two() {
      init();
      let result = json_logic_run(
        r#"{"root": [{"<": [{"var": "a"}, {"var": "b"} ] }] }"#,
        r#"{"a": "1", "b": "2" }"#,
      );
      assert!(result.is_ok());
      assert_eq!(result, Ok(json!(Value::Bool(true))))
    }
  }

  #[test]
  fn test_missing_var() {
    let result = json_logic_run(r#"{"root":[{"var":["missing"]} ]}"#, r#"{}"#);
    assert_eq!(result, Ok(json!(Value::Null)))
  }

  #[test]
  fn test_stop_with_block() {
    init();
    let result = json_logic_run(r#"{"root":[{"block":[{"stop":"missing"}]} ]}"#, r#"{}"#);
    assert_eq!(result, Ok(json!(Value::Null)))
  }

  #[test]
  fn test_stop_without_block() {
    init();
    let result = json_logic_run(r#"{"root":[{"stop":"missing"} ]}"#, r#"{}"#);
    assert_eq!(result, Err("Stopping a block".to_string()))
  }
}