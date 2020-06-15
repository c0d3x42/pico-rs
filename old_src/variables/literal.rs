use serde::{Serialize, Deserialize};

use crate::{Executable, PicoHashMap};

type VarS = String;
type VarN = u32;
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum VarLiteral {
    S (VarS),
    N (VarN)
}
impl VarLiteral {
    pub fn variant_eq(l: &VarLiteral, r: &VarLiteral)->bool{
        match(l,r){
        (&VarLiteral::S(_), &VarLiteral::S(_)) => true,
        (&VarLiteral::N(_), &VarLiteral::N(_)) => true,
        _ => false
        }
    }
    pub fn value_eq(l: &VarLiteral, r: &VarLiteral)->bool{
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

    pub fn eq(&self, other: &VarLiteral) ->bool {
        if( VarLiteral::variant_eq(self, other)){
            debug!("EQ {:?}, {:?}", self, other);
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