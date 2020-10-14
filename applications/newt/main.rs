use picolang::types::{der::RuleFile };
use picolang::types::PicoRule;
use serde_json;

use std::convert::TryFrom;
use log::error;

fn main() {
  env_logger::init();

  let j = r#"
    {
      "version": "1.2",
      "root": [ 
        {
          "set": ["vendors", { "aapl": "Apple", "msft": "Microsoft" }]
        },
        {
          "let": ["vendor_name", {"var":["/aapl"], "register": "vendors"}]
        },
        {
          "let": ["vendor_name_2", {"var":["$.aapl"], "register": "vendors", "type": "path"}]
        },
        {
          "if": [ {"==": ["l","b"]}, {"==": ["one", "two"]}, {"==": [ "three", "four"]}]
        },
        {
          "debug": "dd"
        },
        {
          "if": [ 
            {"==": ["l","b"]}, 
            {"==": ["one", "two"]}, 
            {"==": [ "three", "four"]},
            {"==": [ "five", "six"]}
          ]
        },
        {
          "if": [ 
            {"<": ["1","2","3"]}, 
            {"==":["seven","eight"]}
          ]
        },
        { "set": ["s1", {"yeah": true} ] },
        { "set": ["s2", "yeah"] },
        { "let": ["x", "xxxx"]},
        { "let": ["x2", {"if": [ {"==": [ "a", "b"]}, "kk" ] }]},
        {
          "if": [
            { "==": [ "a", {"var": ["a", "l" ], "type": "pointer", "register": ["_", "k"]  } ]}
          ]
        }

      ]
    }
  "#;

  let rule: RuleFile = serde_json::from_str(&j).unwrap();

  println!("Rule = {:?}", rule);

  match  PicoRule::try_from(rule) {

    Ok(pico_rule) => { 

      println!("RUNNING...");
      pico_rule.run();
      println!("Pico = {:?}", pico_rule);
    },
    Err(err) => error!("Err: {}",err)

  }


}
