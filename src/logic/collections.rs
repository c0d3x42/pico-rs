use serde::{Deserialize, Serialize, Serializer};

use crate::logic::equality::PicoConditionEquality;
use crate::{Executable, PicoHashMap};

#[derive(Serialize, Deserialize, Debug)]
pub struct PicoConditionAnd {
    and: Vec<PicoConditions>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PicoConditionOr {
    or: Vec<PicoConditions>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PicoConditions {
    And(PicoConditionAnd),
    Or(PicoConditionOr),
    Eq(PicoConditionEquality),
}
impl Executable for PicoConditions {
    fn exec(&self, hm: &PicoHashMap) -> bool {
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
impl Executable for PicoConditionAnd {
    fn exec(&self, hm: &PicoHashMap) -> bool {
        debug!("Condition AND {:?}", self);
        let itterator = self.and.iter();
        let mut counter = 0;
        for c in itterator {
            debug!("Itterating [{:?}]", counter);
            counter = counter + 1;
            c.exec(hm);
        }
        let b = self.and.iter().map(|c| c.exec(hm)).collect::<Vec<bool>>();
        debug!("Condition AND ----- booleans {:?}", b);
        return true;
    }
}
impl Executable for PicoConditionOr {
    fn exec(&self, hm: &PicoHashMap) -> bool {
        debug!("Condition OR {:?}", self);

        let b = self.or.iter().map(|c| c.exec(hm)).collect::<Vec<bool>>();
        debug!("Condition OR booleans {:?}", b);

        for c in self.or.iter() {
            c.exec(hm);
        }
        return true;
    }
}
