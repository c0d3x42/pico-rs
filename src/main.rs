
extern crate serde_json;
extern crate serde;
extern crate valico;

use serde_json::{Value,to_string_pretty};
use valico::json_schema;
use std::fs::File;

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
struct VarLookup {
    var: VarLiteral
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum VarLiteral {
    S (String),
    N (u32)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Var {
    Literal(VarLiteral),
    Lookup(VarLookup)
}


#[derive(Serialize, Deserialize, Debug)]
struct PicoConditionEqualityTuple (Var, Var, Var);

#[derive(Serialize, Deserialize, Debug)]
struct PicoConditionEquality {
    #[serde(rename = "==")]
    eq: PicoConditionEqualityTuple
}


#[derive(Serialize, Deserialize, Debug)]
struct PicoConditionAnd {
    and: Vec<PicoConditions>
}

#[derive(Serialize, Deserialize, Debug)]
struct PicoConditionOr {
    or: Vec<PicoConditions>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum PicoConditions {
//    And { and: Vec<String> },
    And(PicoConditionAnd),
    Or(PicoConditionOr),
//    Simple(PicoConditionSimple)
    Eq(PicoConditionEquality)
}


#[derive(Serialize, Deserialize, Debug)]
struct PicoIfThenElseTuple (PicoConditions, String, String);

#[derive(Serialize, Deserialize, Debug)]
struct PicoIfThenElse {
    r#if: PicoIfThenElseTuple
}


fn main() {
    println!("Hello, world!");
    let json_v4_schema: Value = serde_json::from_reader(File::open("schema/schema.json").unwrap()).unwrap();

    println!("schema is {:?}", json_v4_schema);
    let mut scope = json_schema::Scope::new();
    let schema = scope.compile_and_return(json_v4_schema.clone(), false).unwrap();

    println!("Is valid: {}", schema.validate(&json_v4_schema).is_valid());

    let json_rules: PicoIfThenElse = serde_json::from_reader(File::open("pico.json").unwrap()).unwrap();
    println!("Pico rules: {:?}", json_rules)

}