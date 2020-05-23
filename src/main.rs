
extern crate serde_json;
extern crate serde;
extern crate valico;

use serde_json::{Value};
use valico::json_schema;
use std::fs::File;
use std::collections::HashMap;
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
struct PicoConditionEqualityTuple (Var, Var);

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

#[derive(Serialize, Deserialize, Debug )]
#[serde(untagged)]
enum PicoConditionsO {
//    And { and: Vec<String> },
    And(PicoConditionAnd),
//    PicoConditionAnd,
    Or(PicoConditionOr),
//    PicoConditionOr,
//    Simple(PicoConditionSimple)
    Eq(PicoConditionEquality),
//    PicoConditionEquality
}

#[derive(Serialize, Deserialize, Debug )]
#[serde(untagged)]
enum PicoConditions {
    And(PicoConditionAnd),
    Or(PicoConditionOr),
    Eq(PicoConditionEquality)
}


impl Executable for PicoConditions{
    fn exec(&self) -> bool {
        match &*self {
            PicoConditions::And(_x) => { return _x.exec(); },
            PicoConditions::Or(_x) => { return _x.exec();},
            PicoConditions::Eq(_x) => { return _x.exec();},
        }
    }
}


trait Executable {
    fn exec(&self)->bool;
}

impl Executable for PicoConditionAnd {
    fn exec(&self)->bool{
        println!("Condition AND {:?}", self);
        let itterator = self.and.iter();
        let mut counter = 0;
        for c in itterator{
            println!("Itterating [{:?}]", counter);
            counter = counter+1;
            c.exec();
        }
        let b = self.and.iter().map(|c|{ c.exec()}).collect::<Vec<bool>>();
        println!("Condition AND ----- booleans {:?}", b);
        return true;
    }
}
impl Executable for PicoConditionOr {
    fn exec(&self)->bool{
        println!("Condition OR {:?}", self);

        let b = self.or.iter().map(|c|{ c.exec()}).collect::<Vec<bool>>();
        println!("Condition OR booleans {:?}", b);

        for c in self.or.iter(){
            c.exec();
        }
        return true;
    }
}

impl Executable for PicoConditionEquality {
    fn exec(&self)->bool{
        println!("Condition EQ {:?}", self);
        return true;
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PicoIfThenElseTuple (PicoConditions, String, String);

#[derive(Serialize, Deserialize, Debug)]
struct PicoIfThenElse {
    ite: (PicoConditions, String, String)
}

impl PicoIfThenElse {
    pub fn exec(&self, hm:HashMap<String, String>) -> bool {
//        println!("Exec: {:?}", self.r#if.0);
//        let c = &self.r#if;
//        let i = &c.0;
        let i = &self.ite.0;

        i.exec();

        let value = hm.get("lop");
        if let Some(v) =value {
            println!("ALSO found {:?}", v)
        }
        
        println!("Exec IIIIi 0: {:?}", i);
        println!("Exec IIIIi 1: {:?}", self.ite.1);
        println!("Exec IIIIi 2: {:?}", self.ite.2);
        return true;
    }
}

struct ContextVars { hm: HashMap<String,String>}
impl ContextVars{
    fn new()-> ContextVars{
        ContextVars{ hm: HashMap::new()}
    }
}


fn main() {
    println!("Hello, world!");
    let json_v4_schema: Value = serde_json::from_reader(File::open("schema/schema.json").unwrap()).unwrap();

    println!("schema is {:?}", json_v4_schema);
    let mut scope = json_schema::Scope::new();
    let schema = scope.compile_and_return(json_v4_schema.clone(), false).unwrap();

    println!("Is valid: {}", schema.validate(&json_v4_schema).is_valid());

    let json_rules: PicoIfThenElse = serde_json::from_reader(File::open("pico.json").unwrap()).unwrap();
    println!("Pico rules: {:?}", json_rules);

    let mut oo = ContextVars::new();
    oo.hm.insert("bob".to_string(), "boooob".to_string());

    let mut hm :HashMap<String,String> = HashMap::new();
    hm.insert("lop".to_ascii_lowercase(), "bingo".into());
    let value = hm.get("lop");
    if let Some(v) =value {
        println!("FOUND {:?}", v);
    }


    let truth = json_rules.exec( hm);
    println!("Truth: {:?}", truth);

}
