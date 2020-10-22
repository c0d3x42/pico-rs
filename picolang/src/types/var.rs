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
    key_type_f: VarLookupType,

    /**
     * fallback value
     */
    default: PicoValue,

    registers: Vec<String>,
    jmespath: Option<jmespatch::Expression<'static>>,
}

impl Default for ExprVar {
    fn default() -> Self {
        Self {
            key: "/".to_string(),
            key_type: VarKeyType::Simple,
            key_type_f: VarLookupType::Pointer(VarPointer {pointer: String::from("")}),
            default: PicoValue::Null,
            registers: Vec::new(),
            jmespath: None,
        }
    }
}

impl TryFrom<der::VarOp> for ExprVar {
    type Error = PicoRuleError;

    fn try_from(var: der::VarOp) -> Result<ExprVar, Self::Error> {

        trace!("ExprVar::TryFrom var {:?}", var);
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

        match var.r#type {
            der::VarType::Path => {
                v.key_type = VarKeyType::JMESPath;
                v.key_type_f = VarLookupType::JmesPath(VarJmesPath {
                    expr: jmespatch::compile(&v.key).map_err(|err| {
                        error!("JMESPath[{}]: {}", v.key, err);
                        PicoRuleError::InvalidPicoRule
                    })?
                })
            },
            der::VarType::Pointer => {
                v.key_type = VarKeyType::JSONPointer;
                v.key_type_f = VarLookupType::Pointer(VarPointer{pointer: v.key.clone()})
            }
            _ => {}
        }

        /*
        if let der::VarType::Path = var.r#type {
            let path = jsonpath_compile(&v.key);
            v.key_type = VarKeyType::JSONPath;
        }
        */

        Ok(v)
    }
}

impl ExprVar {
    pub fn exec(&self, ctx: &Context) -> Result<PicoValue, PicoRuleError> {
        info!("ExprVar key {} keytype {:?}", self.key, self.key_type);
        let result = match self.key_type {
            VarKeyType::Simple => ctx
                .input_get(&self.key)
                .map_or_else(|| None, |v| Some(v.clone()))
                .ok_or(PicoRuleError::NoSuchVariable {
                    variable: self.key.to_string(),
                }),
            VarKeyType::JMESPath => self.key_type_f.exec(ctx.input()),
            VarKeyType::JSONPointer => {
                match ctx.input().pointer(&self.key) {
                    Some(value) => Ok(value.clone()),
                    None => Ok(PicoValue::Null)
                }
            },
            _ => Err(PicoRuleError::InvalidPicoRule),
        };

        match result {
            Ok(k) => Ok(k),
            Err(err) => match err {
                PicoRuleError::NoSuchVariable { variable } => {
                    info!("ExprVar using default value {}", variable);
                    Ok(self.default.clone())
                }
                _ => Err(err),
            },
        }
    }
}

#[derive(Debug)]
pub struct VarPointer {
    pointer: String
}

#[derive(Debug)]
pub struct VarJmesPath {
    expr: jmespatch::Expression<'static>,
}

#[derive(Debug)]
pub enum VarLookupType {
    Pointer(VarPointer),
    JmesPath(VarJmesPath),
}

impl VarLookupType {
    fn exec(&self, val: &PicoValue) -> Result<PicoValue, PicoRuleError> {
        let result = match self {
            VarLookupType::Pointer(ref pointer) => PicoValue::Null,
            VarLookupType::JmesPath(ref jmes_path) => json!(jmes_path.expr.search(val).unwrap()),
        };
        Ok(result)

        //Err(PicoRuleError::InvalidPicoRule)
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
