
use serde::{Serialize, Deserialize};
use crate::{Executable, VarLookup, PicoHashMap};
use crate::variables::literal::VarLiteral;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Var {
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
    pub fn resolve(&self, hm: &PicoHashMap)->Option<VarLiteral>{
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