use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Number;
use serde_json::Value;

pub type PicoValue = Value;

use crate::commands::execution::{Execution, ExecutionResult, FnResult};
use crate::context::PicoContext;
use crate::errors::PicoError;
use crate::lookups::LookupCommand;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PicoValueOld {
    UnsignedNumber(u64),
    Number(i64),
    String(String),
    Boolean(bool),
    //Array(Vec<PicoValue>),
    //Dictionary(HashMap<String, PicoValue>),
}

/*
impl Execution for PicoValueOld {
    fn name(&self) -> String {
        "PicoValue".to_string()
    }

    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &mut PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> FnResult {
        trace!("pico cloning");
        Ok(ExecutionResult::Continue(self.clone()))
    }
}
*/

impl Execution for PicoValue {
    fn name(&self) -> String {
        "PicoValue".to_string()
    }

    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &mut PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> FnResult {
        trace!("pico cloning");
        Ok(ExecutionResult::Continue(self.clone()))
    }
}

impl fmt::Display for PicoValueOld {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        info!("PicoValue::Display {}, {:?}", self, self);
        return write!(f, "{}", self);
    }
}

impl PartialEq for PicoValueOld {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&PicoValueOld::Boolean(a), &PicoValueOld::Boolean(b)) => a == b,
            (&PicoValueOld::UnsignedNumber(a), &PicoValueOld::UnsignedNumber(b)) => a == b,
            (&PicoValueOld::Number(a), &PicoValueOld::Number(b)) => a == b,
            (&PicoValueOld::String(ref a), &PicoValueOld::String(ref b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for PicoValueOld {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (&PicoValueOld::UnsignedNumber(a), &PicoValueOld::UnsignedNumber(b)) => Some(a.cmp(&b)),
            (&PicoValueOld::Number(a), &PicoValueOld::Number(b)) => Some(a.cmp(&b)),
            (&PicoValueOld::String(ref a), &PicoValueOld::String(ref b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

#[test]
fn eq9() {
    let v = json!(9);
    //assert_eq!(v.eq(&PicoValue::Number(9.0)), true);
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum VarValue {
    Lookup(String),
    DefaultLookup(String, PicoValue),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VarLookup {
    var: VarValue,
}

impl Execution for VarLookup {
    fn name(&self) -> String {
        "VarLookup".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        _runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        match &self.var {
            // Plain lookup in ctx variables
            VarValue::Lookup(s) => {
                debug!("lookup {:?}", s);
                // let lookup = ctx.variables.get(s);
                let lookup = ctx.get_value(s);
                match lookup {
                    Some(v) => {
                        let r = v.clone();
                        Ok(ExecutionResult::Continue(r))
                    }
                    None => {
                        info!("Failed to lookup var {:?}", s);
                        // let local_lookup = ctx.local_variables.get(s);
                        let local_lookup = ctx.get_value(s);
                        match local_lookup {
                            Some(v) => Ok(ExecutionResult::Continue(v.clone())),
                            None => Err(PicoError::NoSuchValue(format!("no such var {}", s))),
                        }
                    }
                }
            }
            VarValue::DefaultLookup(varname, fallback) => {
                debug!("default lookup {:?}, {:?}", varname, fallback);

                //let lookup = ctx.variables.get(varname);
                let lookup = ctx.get_value(varname);
                match lookup {
                    Some(value) => Ok(ExecutionResult::Continue(value.clone())),
                    None => Ok(ExecutionResult::Continue(fallback.clone())),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Var {
    String(String),
    Literal(PicoValue),
    Lookup(VarLookup),
}

impl Execution for Var {
    fn name(&self) -> String {
        "Var".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        match self {
            Var::String(s) => Ok(ExecutionResult::Continue(PicoValue::String(s.to_string()))),
            Var::Lookup(lookup) => lookup.run_with_context(pico_rules, runtime, ctx),
            Var::Literal(literal) => Ok(ExecutionResult::Continue(literal.clone())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ValueProducer {
    Pointer(Pointer),
    Lookup(VarLookup),
    Slice(Slice),
    ConCat(ConCat),
    Extract(Box<Extract>),
    DictionaryLookup(LookupCommand),
    LiteralString(LiteralString),

    UnsupportedObject(PicoValue),
}

impl Execution for ValueProducer {
    fn name(&self) -> String {
        "Var".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        trace!("producer running..");
        match self {
            ValueProducer::Lookup(lookup) => lookup.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::Pointer(pointer) => pointer.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::Slice(slice) => slice.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::ConCat(concat) => concat.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::Extract(extract) => extract.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::LiteralString(ls) => ls.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::UnsupportedObject(literal) => {
                literal.run_with_context(pico_rules, runtime, ctx)
            }
            ValueProducer::DictionaryLookup(dict) => {
                dict.run_with_context(pico_rules, runtime, ctx)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtractInternal(#[serde(with = "serde_regex")] Regex, ValueProducer);

#[derive(Serialize, Deserialize, Debug)]
pub struct Extract {
    extract: ExtractInternal,
}

/*
 * Extract - extracts to context variables named from the regex named capture
 */
impl Execution for Extract {
    fn name(&self) -> String {
        return String::from("extract");
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        let with_value = self.extract.1.run_with_context(pico_rules, runtime, ctx)?;
        match with_value {
            ExecutionResult::Continue(continuation) => match continuation {
                PicoValue::String(string_value) => {
                    let captures = self.extract.0.captures(&string_value);
                    info!("CCCCCCC {:?}", captures);

                    if let Some(caps) = &captures {
                        let dict: HashMap<String, PicoValue> = self
                            .extract
                            .0
                            .capture_names()
                            .flatten()
                            .filter_map(|n| {
                                Some((
                                    String::from(n),
                                    PicoValue::String(caps.name(n)?.as_str().to_string()),
                                ))
                            })
                            .collect();
                        info!("DICT = {:?}", dict);
                        return Ok(ExecutionResult::Setting(dict));
                    }
                    Ok(ExecutionResult::Setting(HashMap::new()))
                }
                _ => Err(PicoError::IncompatibleComparison),
            },
            everything_else => Ok(everything_else),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConCat {
    concat: Vec<ValueProducer>,
}

impl Execution for ConCat {
    fn name(&self) -> String {
        "concat".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        let words = &self
            .concat
            .iter()
            .map(|e| e.run_with_context(pico_rules, runtime, ctx))
            .filter(|x| x.is_ok())
            .filter_map(Result::ok)
            .filter_map(|p| match p {
                ExecutionResult::Continue(c) => Some(c),
                _ => None,
            })
            .collect::<Vec<PicoValue>>();

        info!("TTTTT {:?}", words);
        for word in words {
            trace!("concat word {:?}", word);
        }

        let tt = words
            .iter()
            .filter_map(|m| match m {
                PicoValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Vec<String>>();

        info!("TT {:?}", tt.join(""));

        Ok(ExecutionResult::Continue(PicoValue::String(tt.join(""))))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Slice {
    slice: (Box<ValueProducer>, isize, Option<isize>),
}

fn slice_starts_at(requested_start: isize, vec_length: usize) -> usize {
    if requested_start < 0 {
        // index backwards from the end
        let end_result = usize::try_from(requested_start.abs());
        let end_pos = match end_result {
            Ok(value) => value,
            Err(_e) => usize::max_value(),
        };
        if end_pos < vec_length {
            vec_length - end_pos
        } else {
            0
        }
    } else {
        let start_result = usize::try_from(requested_start);
        let start_offset: usize = match start_result {
            Ok(value) => value,
            Err(_e) => usize::max_value(),
        };

        if start_offset > vec_length {
            vec_length
        } else {
            start_offset
        }
    }
}

fn slice_ends_at(requested_end: isize, vec_length: usize) -> usize {
    if requested_end < 0 {
        let end_result = usize::try_from(requested_end.abs());
        let end_offset = match end_result {
            Ok(value) => value,
            Err(_e) => usize::max_value(),
        };

        if end_offset < vec_length {
            vec_length - end_offset
        } else {
            vec_length
        }
    } else {
        let end_result = usize::try_from(requested_end);
        let end_offset: usize = match end_result {
            Ok(value) => value,
            Err(_e) => usize::max_value(),
        };

        if end_offset > vec_length {
            vec_length
        } else {
            vec_length
        }
    }
}

impl Execution for Slice {
    fn name(&self) -> String {
        "slice".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        info!("slicing");
        let s = self.slice.0.run_with_context(pico_rules, runtime, ctx)?;
        let start_index = self.slice.1;
        //let endIndex = self.slice.2;

        let final_result = match s {
            ExecutionResult::Continue(r) => match r {
                PicoValue::String(matched_string) => {
                    trace!("Slicing string {:?}", matched_string);

                    let my_vec: Vec<char> = matched_string.chars().collect();
                    trace!("sliced vec {:?}", my_vec);

                    let iistart = slice_starts_at(start_index, my_vec.len());
                    trace!("IIII start {:?}", iistart);

                    let end_offset = match self.slice.2 {
                        Some(ending) => slice_ends_at(ending, my_vec.len()),
                        None => my_vec.len(),
                    };

                    if let Some(substring) = matched_string.get(iistart..end_offset) {
                        return Ok(ExecutionResult::Continue(PicoValue::String(
                            substring.to_string(),
                        )));
                    }

                    return Ok(ExecutionResult::Continue(PicoValue::String("".to_string())));
                }
                _ => Err(PicoError::IncompatibleComparison),
            },
            everything_else => Ok(everything_else),
        };

        return final_result;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pointer {
    pointer: String, // JSON pointer
}

impl Execution for Pointer {
    fn name(&self) -> String {
        "slice".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        info!("consulting pointer");
        if let Some(json) = &ctx.json {
            trace!("we have some json, checking pointer {}", self.pointer);
            if let Some(value) = json.pointer(&self.pointer) {
                trace!("found some value {}", value);
                return Ok(ExecutionResult::Continue(PicoValue::String(
                    value.to_string(),
                )));
            }
        }

        Ok(ExecutionResult::Continue(PicoValue::Bool(true)))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LiteralString(String);

impl Execution for LiteralString {
    fn name(&self) -> String {
        "literalstring".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        info!("HIT a literal string {}", self.0);

        Ok(ExecutionResult::Continue(PicoValue::String(
            self.0.to_string(),
        )))
    }
}

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct LiteralNumber(Number);

impl Execution for LiteralNumber {
    fn name(&self) -> String {
        "liternalnumber".to_string()
    }

    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &mut PicoRuntime,
        ctx: &mut PicoContext,
    ) -> FnResult {
        info!("HIT a literal number {}", self.0);

        Ok(ExecutionResult::Continue(PicoValue::Number(json!(self.0))))
    }
}
*/
