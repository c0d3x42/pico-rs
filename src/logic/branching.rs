use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

use crate::logic::collections::PicoConditions;
use crate::logic::equality::PicoConditionEquality;
use crate::pathing::path::PathLookup;
use crate::{Executable, PicoHashMap};

#[derive(Serialize, Deserialize, Debug)]
struct PicoIfThenElseTuple(PicoConditions, String, String);

#[derive(Serialize, Deserialize, Debug)]
pub struct PicoIfThenElse {
    ite: (PicoConditions, String, String),
    p: PathLookup,
}

impl PicoIfThenElse {
    pub fn exec(&self, hm: HashMap<String, String>) -> bool {
        //        println!("Exec: {:?}", self.r#if.0);
        //        let c = &self.r#if;
        //        let i = &c.0;
        let i = &self.ite.0;

        i.exec(&hm);

        let value = hm.get("lop");
        if let Some(v) = value {
            debug!("ALSO found {:?}", v)
        }
        debug!("Exec IIIIi 0: {:?}", i);
        debug!("Exec IIIIi 1: {:?}", self.ite.1);
        debug!("Exec IIIIi 2: {:?}", self.ite.2);
        return true;
    }
}
