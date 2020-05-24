
extern crate serde_json;
extern crate serde;
extern crate valico;

use serde_json::{Value};
use valico::json_schema;
use std::fs::File;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

mod context;
use crate::context::{pico::PicoContext, pico::PicoHashMap};


#[derive(Serialize, Deserialize, Debug)]
struct VarLookup {
    var: VarLiteral
}
impl Executable for VarLookup {
    fn exec (&self, hm: &PicoHashMap)->bool{
        println!("VarLookup");
        return self.var.exec(hm);
    }
}

type VarS = String;
type VarN = u32;
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum VarLiteral {
    S (VarS),
    N (VarN)
}
impl VarLiteral {
    fn variant_eq(l: &VarLiteral, r: &VarLiteral)->bool{
        match(l,r){
        (&VarLiteral::S(_), &VarLiteral::S(_)) => true,
        (&VarLiteral::N(_), &VarLiteral::N(_)) => true,
        _ => false
        }
    }
    fn value_eq(l: &VarLiteral, r: &VarLiteral)->bool{
        match(l,r){
            (VarLiteral::S(lv), VarLiteral::S(rv)) => {
                return lv == rv;
            },
            (VarLiteral::N(lv), VarLiteral::N(rv)) => {
                return lv == rv;
            },
            _ => false
        }
    }
}

impl Executable for VarLiteral {
    fn exec (&self, hm: &PicoHashMap)->bool{

        match &*self {
            Self::S(_s) => {
                println!("VarLiteral (S) {:?}", _s);
            },
            Self::N(_n) => {
                println!("VarLiteral (N) {:?}", _n);
            },

        }
        return true;
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Var {
    Literal(VarLiteral),
    Lookup(VarLookup)
}

impl Executable for Var{

    fn exec(&self, hm: &PicoHashMap)->bool{
        match &*self {
            Self::Literal(varLiteral) => { return varLiteral.exec(hm)},
            Self::Lookup(varLookup) => { return varLookup.exec(hm)},

        }
    }
}
impl Var {
    fn resolve(&self, hm: &PicoHashMap)->Option<VarLiteral>{
        match &*self {
            Self::Literal(v) => {

                match v {
                    VarLiteral::N(n) => {
                        return Some(VarLiteral::N(*n));
                    }
                    VarLiteral::S(s) => {
                        let t = s.clone();
                        return Some(VarLiteral::S(t));
                    }
                }
            },
            Self::Lookup(v) => {
                match &v.var{
                    VarLiteral::S(_s) => {

                        
                        let looked_up = hm.get(&_s.to_string());
                        if let Some(v) = looked_up {
                            let y = VarLiteral::S("lll".to_string());
                            let yy = v;

                            return Some(VarLiteral::S(v.to_string()));
                        } else {
                            return None;
                        }
                    },
                    VarLiteral::N(_n) => {
                        return None;
                    }
                }
            }

        }
    }
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
enum PicoConditions {
    And(PicoConditionAnd),
    Or(PicoConditionOr),
    Eq(PicoConditionEquality)
}


impl Executable for PicoConditions{
    fn exec(&self, hm: &PicoHashMap) -> bool {
        match &*self {
            PicoConditions::And(_x) => { return _x.exec(hm); },
            PicoConditions::Or(_x) => { return _x.exec(hm);},
            PicoConditions::Eq(_x) => { return _x.exec(hm);},
        }
    }
}


trait Executable {
    fn exec(&self, hm: &PicoHashMap)->bool;
}

impl Executable for PicoConditionAnd {
    fn exec(&self, hm: &PicoHashMap)->bool{
        println!("Condition AND {:?}", self);
        let itterator = self.and.iter();
        let mut counter = 0;
        for c in itterator{
            println!("Itterating [{:?}]", counter);
            counter = counter+1;
            c.exec(hm);
        }
        let b = self.and.iter().map(|c|{ c.exec(hm)}).collect::<Vec<bool>>();
        println!("Condition AND ----- booleans {:?}", b);
        return true;
    }
}
impl Executable for PicoConditionOr {
    fn exec(&self, hm: &PicoHashMap)->bool{
        println!("Condition OR {:?}", self);

        let b = self.or.iter().map(|c|{ c.exec(hm)}).collect::<Vec<bool>>();
        println!("Condition OR booleans {:?}", b);

        for c in self.or.iter(){
            c.exec(hm);
        }
        return true;
    }
}

impl Executable for PicoConditionEquality {
    fn exec(&self, hm: &PicoHashMap)->bool{
        println!("Condition EQ {:?}", self);
        let lhs = &self.eq.0.exec(hm);
        let rhs = &self.eq.1.exec(hm);

        let lhsv = &self.eq.0.resolve(hm);
        let rhsv = &self.eq.1.resolve(hm);

        println!("LHSV: {:?}, RHSV: {:?}", lhsv, rhsv);

        if let Some(lv) = lhsv{
            println!("EQUALITY left {:?}", lv);
            if let Some(rv) = rhsv{
                println!("EQUALITY right {:?}", rv);
                if ! VarLiteral::variant_eq(&lv, &rv){
                    println!("VARIANT NOT THE SAME - NO COMPARE");
                    return false;
                }
                let b = VarLiteral::value_eq(&lv, &rv);
                println!("EQUALITY is {:?}", b);
                return b;
            }
        } else {
            return false;
        }

        return lhs == rhs;
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

        i.exec(&hm);

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
    oo.hm.insert("lop".to_string(), "LOOOOB".to_string());

    let mut hm :HashMap<String,String> = HashMap::new();
    hm.insert("lop".to_ascii_lowercase(), "bingo".into());
    let value = hm.get("lop");
    if let Some(v) =value {
        println!("FOUND {:?}", v);
    }

    let t = PicoContext::new();
    println!("PC {:?}", t);
    let got = t.get("lop".to_string());


    let truth = json_rules.exec( t.values );
    println!("Truth: {:?}", truth);

    let a  = VarLiteral::S("lll".to_string());

}
