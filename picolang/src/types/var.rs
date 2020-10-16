use super::*;

use jsonpath_lib::{compile as jsonpath_compile, JsonPathError};
use serde_json::Value;

use jmespatch;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

use crate::PicoValue;


#[derive(Debug)]
pub enum VarKeyType {
    Simple,
    JSONPointer,
    JSONPath,
    /**
     * https://lib.rs/crates/jmespatch
     */
    JMESPath,
    /**
     * https://crates.io/crates/json_dotpath
     */
    JSONDotPath,
    /**
     * https://crates.io/crates/jsondata
     */
    JSONData,
}

#[derive(Debug)]
pub struct ExprVar {
    key: String,
    key_type: VarKeyType,
    default: PicoValue,

    registers: Vec<String>,
    jmespath: Option<jmespatch::Expression<'static>>,
}

impl Default for ExprVar {
    fn default() -> Self {
        Self {
            key: "/".to_string(),
            key_type: VarKeyType::Simple,
            default: PicoValue::Null,
            registers: Vec::new(),
            jmespath: None,
        }
    }
}

impl TryFrom<der::VarOp> for ExprVar {
    type Error = PicoRuleError;

    fn try_from(var: der::VarOp) -> Result<ExprVar, Self::Error> {
        let mut v: Self = Self {
            ..Default::default()
        };

        let j = jmespatch::compile("bar").unwrap();
        v.jmespath = Some(j);

        match var.value {
            der::VarValue::String(s) => v.key = s,
            der::VarValue::OneString(s) => {
                if let Some(s1) = s.first() {
                    v.key = s1.to_owned();
                }
            }

            der::VarValue::WithDefault(s, d) => {
                v.key = s;
                v.default = d;
            }
        }

        match var.register {
            der::VarRegister::Single(s) => v.registers.push(s),
            der::VarRegister::Named(registers) => v.registers = registers,
            _ => {}
        }

        if let der::VarType::Path = var.r#type {
            let path = jsonpath_compile(&v.key);
            v.key_type = VarKeyType::JSONPath;
        }

        Ok(v)
    }
}


impl ExprVar {
    pub fn exec(&self, ctx: &Context) -> Result<PicoValue, PicoRuleError> {

        info!("ExprVar key {} keytype {:?}", self.key, self.key_type);
        let result = match self.key_type {
            VarKeyType::Simple => {
                ctx.input_get(&self.key)
                .map_or_else(
                    ||None, 
                    |v| Some(v.clone()))
                .ok_or(PicoRuleError::NoSuchVariable{ variable: self.key.to_string()})
            },
            _ => Err(PicoRuleError::InvalidPicoRule),
        };

        match result {
            Ok(k) => Ok(k),
            Err( err) => match err {
                PicoRuleError::NoSuchVariable{variable} => {
                    info!("ExprVar using default value {}", variable);
                    Ok(self.default.clone())},
                _ => Err(err)
            }
        }
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn make_pico_var() {
        let varOp: der::VarOp = der::VarOp {
            value: ("l".to_string(), None),
            register: None,
            path: false
        };

        let picoVar = PicoVarOp::from(varOp);
    }
}
*/
