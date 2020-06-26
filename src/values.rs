use serde::{Deserialize, Serialize};

use crate::command::{Execution, ExecutionResult, FnResult};
use crate::context::{Context, VariablesMap};
use crate::errors::PicoError;
use itertools::Itertools;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PicoValue {
    UnsignedNumber(usize),
    Number(isize),
    String(String),
    Boolean(bool),
}

impl Execution for PicoValue {
    fn name(&self) -> String {
        return "PicoValue".to_string();
    }

    fn run_with_context(&self, _ctx: &mut Context) -> FnResult {
        trace!("pico cloning");
        return Ok(ExecutionResult::Continue(self.clone()));
    }
}

impl fmt::Display for PicoValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        info!("PicoValue::Display {}, {:?}", self, self);
        return write!(f, "{}", self);
    }
}

impl PartialEq for PicoValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&PicoValue::Boolean(a), &PicoValue::Boolean(b)) => a == b,
            (&PicoValue::UnsignedNumber(a), &PicoValue::UnsignedNumber(b)) => a == b,
            (&PicoValue::Number(a), &PicoValue::Number(b)) => a == b,
            (&PicoValue::String(ref a), &PicoValue::String(ref b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for PicoValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (&PicoValue::UnsignedNumber(a), &PicoValue::UnsignedNumber(b)) => Some(a.cmp(&b)),
            (&PicoValue::Number(a), &PicoValue::Number(b)) => Some(a.cmp(&b)),
            (&PicoValue::String(ref a), &PicoValue::String(ref b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

#[test]
fn eq9() {
    let v = PicoValue::Number(9);
    assert_eq!(v.eq(&PicoValue::Number(9)), true);
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
        return "VarLookup".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        match &self.var {
            // Plain lookup in ctx variables
            VarValue::Lookup(s) => {
                debug!("lookup {:?}", s);
                // let lookup = ctx.variables.get(s);
                let lookup = ctx.getValue(s);
                match lookup {
                    Some(v) => {
                        let r = v.clone();
                        return Ok(ExecutionResult::Continue(r));
                    }
                    None => {
                        info!("Failed to lookup var {:?}", s);
                        // let local_lookup = ctx.local_variables.get(s);
                        let local_lookup = ctx.getValue(s);
                        match local_lookup {
                            Some(v) => return Ok(ExecutionResult::Continue(v.clone())),
                            None => return Err(PicoError::NoSuchValue),
                        }
                    }
                };
            }
            VarValue::DefaultLookup(varname, fallback) => {
                debug!("default lookup {:?}, {:?}", varname, fallback);

                //let lookup = ctx.variables.get(varname);
                let lookup = ctx.getValue(varname);
                match lookup {
                    Some(value) => return Ok(ExecutionResult::Continue(value.clone())),
                    None => return Ok(ExecutionResult::Continue(fallback.clone())),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Var {
    Literal(PicoValue),
    Lookup(VarLookup),
}

impl Execution for Var {
    fn name(&self) -> String {
        return "Var".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        match self {
            Var::Lookup(lookup) => lookup.run_with_context(ctx),
            Var::Literal(literal) => Ok(ExecutionResult::Continue(literal.clone())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ValueProducer {
    Literal(PicoValue),
    Lookup(VarLookup),
    Slice(Slice),
    ConCat(ConCat),
}

impl Execution for ValueProducer {
    fn name(&self) -> String {
        return "Var".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        trace!("producer running..");
        match self {
            ValueProducer::Lookup(lookup) => lookup.run_with_context(ctx),
            ValueProducer::Literal(literal) => literal.run_with_context(ctx),
            ValueProducer::Slice(slice) => slice.run_with_context(ctx),
            ValueProducer::ConCat(concat) => concat.run_with_context(ctx),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConCat {
    concat: Vec<ValueProducer>,
}

impl Execution for ConCat {
    fn name(&self) -> String {
        return "concat".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        let words = &self
            .concat
            .iter()
            .map(|e| e.run_with_context(ctx))
            .filter(|x| x.is_ok())
            .filter_map(Result::ok)
            .filter_map(|p| match p {
                ExecutionResult::Continue(c) => Some(c),
                _ => None,
            })
            .collect::<Vec<PicoValue>>();

        info!("TTTTT {:?}", words);
        for word in words {
            info!("TTTTT word {:?}", word);
        }

        let tt = words
            .iter()
            .filter_map(|m| match m {
                PicoValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Vec<String>>();

        info!("TT {:?}", tt.join(""));

        return Ok(ExecutionResult::Continue(PicoValue::String(tt.join(""))));
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Slice {
    slice: (Box<ValueProducer>, isize, Option<isize>),
}

fn sliceStartsAt(requested_start: isize, vec_length: usize) -> usize {
    if requested_start < 0 {
        // index backwards from the end
        let end_result = usize::try_from(requested_start.abs());
        let end_pos = match end_result {
            Ok(value) => value,
            Err(_e) => usize::max_value(),
        };
        if end_pos < vec_length {
            return vec_length - end_pos;
        } else {
            return 0;
        }
    } else {
        let start_result = usize::try_from(requested_start);
        let start_offset: usize = match start_result {
            Ok(value) => value,
            Err(_e) => usize::max_value(),
        };

        if start_offset > vec_length {
            return vec_length;
        } else {
            return start_offset;
        }
    }
}

fn sliceEndsAt(requested_end: isize, vec_length: usize) -> usize {
    if requested_end < 0 {
        let end_result = usize::try_from(requested_end.abs());
        let end_offset = match end_result {
            Ok(value) => value,
            Err(_e) => usize::max_value(),
        };

        if end_offset < vec_length {
            return vec_length - end_offset;
        } else {
            return vec_length;
        }
    } else {
        let end_result = usize::try_from(requested_end);
        let end_offset: usize = match end_result {
            Ok(value) => value,
            Err(_e) => usize::max_value(),
        };

        if end_offset > vec_length {
            return vec_length;
        } else {
            return vec_length;
        }
    }
}

impl Execution for Slice {
    fn name(&self) -> String {
        return "slice".to_string();
    }

    fn run_with_context(&self, ctx: &mut Context) -> FnResult {
        info!("slicing");
        let s = self.slice.0.run_with_context(ctx)?;
        let startIndex = self.slice.1;
        let endIndex = self.slice.2;

        let final_result = match s {
            ExecutionResult::Continue(r) => match r {
                PicoValue::String(matched_string) => {
                    trace!("Slicing string {:?}", matched_string);

                    let my_vec: Vec<char> = matched_string.chars().collect();
                    trace!("sliced vec {:?}", my_vec);

                    let iistart = sliceStartsAt(startIndex, my_vec.len());
                    trace!("IIII start {:?}", iistart);

                    let end_offset = match self.slice.2 {
                        Some(ending) => sliceEndsAt(ending, my_vec.len()),
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
