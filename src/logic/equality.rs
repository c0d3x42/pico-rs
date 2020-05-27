extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::variables::literal::VarLiteral;
use crate::variables::var::Var;
use crate::{Executable, PicoHashMap};

#[derive(Serialize, Deserialize, Debug)]
pub struct PicoConditionEqualityTuple(Var, Var);

#[derive(Serialize, Deserialize, Debug)]
pub struct VarLiteralTuple(VarLiteral, VarLiteral);

#[derive(Serialize, Deserialize, Debug)]
pub struct PicoConditionEquality {
    #[serde(rename = "==")]
    pub eq: PicoConditionEqualityTuple,
}

impl PicoConditionEquality {
    pub fn resolve(&self, hm: &PicoHashMap) -> Option<VarLiteralTuple> {
        if let Some(l) = self.eq.0.resolve(hm) {
            if let Some(r) = self.eq.1.resolve(hm) {
                return Some(VarLiteralTuple(l, r));
            }
        }
        return None;
    }

    pub fn eq(&self, hm: &PicoHashMap) -> bool {
        if let Some(l) = self.resolve(hm) {
            debug!("HHHHHHHHH resolved");
            let left = l.0;
            let right = &l.1;

            debug!("HHHHHHHHHHH {:?}, {:?}", left, right);

            if left.eq(right) {
                return true;
            }
        }

        return false;
    }
}

impl Executable for PicoConditionEquality {
    fn exec(&self, hm: &PicoHashMap) -> bool {
        debug!("Condition EQ {:?}", self);

        let b = self.eq(hm);
        debug!("HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH {:?}", b);

        let lhs = &self.eq.0.exec(hm);
        let rhs = &self.eq.1.exec(hm);

        let lhsv = &self.eq.0.resolve(hm);
        let rhsv = &self.eq.1.resolve(hm);

        debug!("LHSV: {:?}, RHSV: {:?}", lhsv, rhsv);

        if let Some(lv) = lhsv {
            debug!("EQUALITY left {:?}", lv);
            if let Some(rv) = rhsv {
                debug!("EQUALITY right {:?}", rv);
                if !VarLiteral::variant_eq(&lv, &rv) {
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
