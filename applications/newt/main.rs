use picolang::types::{der::RuleFile, PicoRule};
use serde_json;

fn main() {
  let j = r#"
    {
      "version": "1.2",
      "root": [ 
        {
          "if": [ {"==": ["l","b"]}, {"==": ["one", "two"]}, {"==": [ "three", "four"]}]
        },
        {
          "if": [ {"!=": ["l","b"]}]
        },
        { "if": [ {"var": "l"}] },
        { "if": [ {"var": ["l", "k"]}] },
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
        { "let": ["x", "xxxx"]},
        { "let": ["x2", {"if": [ {"==": [ "a", "b"]}, "kk" ] }]}
      ]
    }
  "#;

  let rule: RuleFile = serde_json::from_str(&j).unwrap();

  println!("Rule = {:?}", rule);

  let pico_rule = PicoRule::from(rule);

  println!("Pico = {:?}", pico_rule);
}
