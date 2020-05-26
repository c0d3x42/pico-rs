
extern crate serde_json;
extern crate serde;
extern crate valico;
extern crate jsonpath_lib as jsonpath;

#[macro_use]
extern crate log;


use serde_json::{Value,json};
use valico::json_schema;
use std::fs::File;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

mod context;
use crate::context::{pico::PicoContext, pico::PicoHashMap};

mod pathing;
use crate::pathing::{path::PathLookup, path::PathLookupAugmented};

trait Initializable {
    fn init(&self)->bool{
        return true;
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct VarLookup {
    var: VarLiteral
}
impl Executable for VarLookup {
    fn exec (&self, hm: &PicoHashMap)->bool{
        debug!("VarLookup");
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

    fn eq(&self, other: &VarLiteral) ->bool {
        if( VarLiteral::variant_eq(self, other)){
            return VarLiteral::value_eq(self, other);
        }
        return false;
    }
}

impl Executable for VarLiteral {
    fn exec (&self, hm: &PicoHashMap)->bool{

        match &*self {
            Self::S(_s) => {
                debug!("VarLiteral (S) {:?}", _s);
            },
            Self::N(_n) => {
                debug!("VarLiteral (N) {:?}", _n);
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
        match self {
            Self::Literal(varLiteral) => { return varLiteral.exec(hm)},
            Self::Lookup(varLookup) => { return varLookup.exec(hm)},

        }
    }
}
impl Var {
    fn resolve(&self, hm: &PicoHashMap)->Option<VarLiteral>{
        match self {
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

struct VarLiteralTuple (VarLiteral, VarLiteral);

#[derive(Serialize, Deserialize, Debug)]
struct PicoConditionEquality {
    #[serde(rename = "==")]
    eq: PicoConditionEqualityTuple
}

impl PicoConditionEquality{

    fn resolve(&self, hm: &PicoHashMap) -> Option<VarLiteralTuple>{
        if let Some(l) = self.eq.0.resolve(hm){
            if let Some(r) = self.eq.1.resolve(hm){
                 return Some(VarLiteralTuple( l,r ));
            }
        }
        return None;
    }

    fn eq(&self, hm: &PicoHashMap) -> bool{
        if let Some(l) = self.resolve(hm){
            debug!("HHHHHHHHH resolved");
            let left = l.0;
            let right = &l.1;

            debug!("HHHHHHHHHHH {:?}, {:?}", left, right);

            if left.eq(right){
                return true;
            }
        }

        return false;
    }
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
impl Executable for PicoConditions {
    fn exec(&self, hm: &PicoHashMap) -> bool{
        match self {
            PicoConditions::And(x) => x.exec(hm),
            PicoConditions::Or(x) => x.exec(hm),
            PicoConditions::Eq(x) => x.exec(hm),
        }
    }
}


/*
impl Executable for PicoConditions{
    fn exec(&self, hm: &PicoHashMap) -> bool {
        match &*self {
            PicoConditions::And(_x) => { return _x.exec(hm); },
            PicoConditions::Or(_x) => { return _x.exec(hm);},
            PicoConditions::Eq(_x) => { return _x.exec(hm);},
        }
    }
}
*/


trait Executable {
    fn exec(&self, _hm: &PicoHashMap)->bool{
        return true;
    }
}

impl Executable for PicoConditionAnd {
    fn exec(&self, hm: &PicoHashMap)->bool{
        debug!("Condition AND {:?}", self);
        let itterator = self.and.iter();
        let mut counter = 0;
        for c in itterator{
            debug!("Itterating [{:?}]", counter);
            counter = counter+1;
            c.exec(hm);
        }
        let b = self.and.iter().map(|c|{ c.exec(hm)}).collect::<Vec<bool>>();
        debug!("Condition AND ----- booleans {:?}", b);
        return true;
    }
}
impl Executable for PicoConditionOr {
    fn exec(&self, hm: &PicoHashMap)->bool{
        debug!("Condition OR {:?}", self);

        let b = self.or.iter().map(|c|{ c.exec(hm)}).collect::<Vec<bool>>();
        debug!("Condition OR booleans {:?}", b);

        for c in self.or.iter(){
            c.exec(hm);
        }
        return true;
    }
}

impl Executable for PicoConditionEquality {
    fn exec(&self, hm: &PicoHashMap)->bool{
        debug!("Condition EQ {:?}", self);

        let b = self.eq(hm); 
        debug!("HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH {:?}", b);
        

        let lhs = &self.eq.0.exec(hm);
        let rhs = &self.eq.1.exec(hm);

        let lhsv = &self.eq.0.resolve(hm);
        let rhsv = &self.eq.1.resolve(hm);

        debug!("LHSV: {:?}, RHSV: {:?}", lhsv, rhsv);

        if let Some(lv) = lhsv{
            debug!("EQUALITY left {:?}", lv);
            if let Some(rv) = rhsv{
                debug!("EQUALITY right {:?}", rv);
                if ! VarLiteral::variant_eq(&lv, &rv){
                    debug!("VARIANT NOT THE SAME - NO COMPARE");
                    return false;
                }
                let b = VarLiteral::value_eq(&lv, &rv);
                debug!("EQUALITY is {:?}", b);
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
    ite: (PicoConditions, String, String),
    p: PathLookup
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
            debug!("ALSO found {:?}", v)
        }
        
        debug!("Exec IIIIi 0: {:?}", i);
        debug!("Exec IIIIi 1: {:?}", self.ite.1);
        debug!("Exec IIIIi 2: {:?}", self.ite.2);
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
    env_logger::init();

    info!("Starting up");

    debug!("Hello, world!");
    let json_v4_schema: Value = serde_json::from_reader(File::open("schema/schema.json").unwrap()).unwrap();

    debug!("schema is {:?}", json_v4_schema);
    let mut scope = json_schema::Scope::new();
    let schema = scope.compile_and_return(json_v4_schema.clone(), false).unwrap();

    debug!("Is valid: {}", schema.validate(&json_v4_schema).is_valid());

    let json_rules: PicoIfThenElse = serde_json::from_reader(File::open("pico.json").unwrap()).unwrap();
    debug!("Pico rules: {:?}", json_rules);

    let mut oo = ContextVars::new();
    oo.hm.insert("bob".to_string(), "boooob".to_string());
    oo.hm.insert("lop".to_string(), "LOOOOB".to_string());

    let mut hm :HashMap<String,String> = HashMap::new();
    hm.insert("lop".to_ascii_lowercase(), "bingo".into());
    let value = hm.get("lop");
    if let Some(v) =value {
        debug!("FOUND {:?}", v);
    }

    let t = PicoContext::new();
    debug!("PC {:?}", t);
    let got = t.get("lop");
    if let Some(vvv) =got {
        debug!("VVVV: {:?}", vvv);
    }


    let truth = json_rules.exec( t.values );
    debug!("Truth: {:?}", truth);

    let a  = VarLiteral::S("lll".to_string());
    let json_obj =json!({ "store": {}});
    let mut selector = jsonpath::selector(&json_obj);
    let t = selector("$.");

    debug!("finish");
    warn!("DONE");
}
