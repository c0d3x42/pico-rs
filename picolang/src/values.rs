use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

/// The base value that can be used.
/// For now identical to [`serde_json::Value`](crate::serde_json::Value)
pub type PicoValue = Value;

use crate::commands::execution::{ValueExecution, ValueResult};
use crate::context::PicoContext;
use crate::errors::PicoError;
use crate::rules::{lookups::LookupType, PicoRules};
use crate::runtime::PicoRuntime;
use regex::Regex;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

impl ValueExecution for PicoValue {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> ValueResult {
        trace!("pico cloning");
        Ok(self.clone())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum VarValue {
    /// String to lookup
    Lookup(String),
    /// String to lookup, with a default value
    DefaultLookup(String, PicoValue),
}

///
/// Getting a PicoValue
#[derive(Serialize, Deserialize, Debug)]
pub struct VarLookup {
    /// String
    /// String, [`PicoValue`](crate::values::PicoValue)
    var: VarValue,
}

impl ValueExecution for VarLookup {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ValueResult {
        match &self.var {
            // Plain lookup in ctx variables
            VarValue::Lookup(s) => {
                trace!("lookup {:?}", s);
                let lookup = ctx.get_value(s);
                match lookup {
                    Some(v) => Ok(v.clone()),
                    None => Err(PicoError::NoSuchValue(format!("no such var {}", s))),
                }
            }
            VarValue::DefaultLookup(varname, fallback) => {
                debug!("default lookup {:?}, {:?}", varname, fallback);

                //let lookup = ctx.variables.get(varname);
                let lookup = ctx.get_value(varname);
                match lookup {
                    Some(value) => Ok(value.clone()),
                    None => Ok(fallback.clone()),
                }
            }
        }
    }
}

/// Types of Var's that can be used
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Var {
    /// A literal String
    String(String),
    /// A literal PicoValue
    Literal(PicoValue),
    /// A named PicoValue to lookup
    Lookup(VarLookup),
}

impl ValueExecution for Var {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ValueResult {
        match self {
            Var::String(s) => Ok(PicoValue::String(s.to_string())),
            Var::Lookup(lookup) => lookup.run_with_context(pico_rules, runtime, ctx),
            Var::Literal(literal) => Ok(literal.clone()),
        }
    }
}

/// All things that can produce a PicoValue
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ValueProducer {
    /// a JSON pointer
    Pointer(Pointer),
    /// A Var
    VarLookup(VarLookup),
    /// Lookup in a table in a PicoRule file
    TableLookup(TableLookup),

    /// Slice of a String
    Slice(Slice),

    /// concatenation of String's
    ConCat(ConCat),
    Extract(Box<Extract>),

    LiteralString(LiteralString),
    LiteralI64(LiteralI64),

    UnsupportedObject(PicoValue),
}

/// Produce a PicoValue
impl ValueExecution for ValueProducer {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ValueResult {
        trace!("producer running..");
        match self {
            ValueProducer::VarLookup(lookup) => lookup.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::Pointer(pointer) => pointer.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::Slice(slice) => slice.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::ConCat(concat) => concat.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::Extract(extract) => extract.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::LiteralString(ls) => ls.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::LiteralI64(i) => i.run_with_context(pico_rules, runtime, ctx),
            ValueProducer::UnsupportedObject(literal) => {
                literal.run_with_context(pico_rules, runtime, ctx)
            }
            ValueProducer::TableLookup(lookup) => lookup.run_with_context(pico_rules, runtime, ctx),
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
impl ValueExecution for Extract {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ValueResult {
        let with_value = self.extract.1.run_with_context(pico_rules, runtime, ctx)?;

        match with_value {
            PicoValue::String(s) => {
                let captures = self.extract.0.captures(&s);
                if let Some(caps) = &captures {
                    let dict: HashMap<String, PicoValue> = self
                        .extract
                        .0
                        .capture_names()
                        .flatten()
                        .filter_map(|n| {
                            Some((
                                String::from(n),
                                // FIXME unwrap may panic
                                PicoValue::String(caps.name(n).unwrap().as_str().to_string()),
                            ))
                        })
                        .collect();
                    Ok(json!(dict))
                } else {
                    Ok(json!({}))
                }
            }
            _ => Err(PicoError::IncompatibleComparison(
                with_value,
                PicoValue::Null,
            )),
        }
    }
}

/// ConCat from a JSON array of other ValueProducers that produce Strings
#[derive(Serialize, Deserialize, Debug)]
pub struct ConCat {
    /// Array of [`ValueProducer`](crate::values::ValueProducer)
    concat: Vec<ValueProducer>,
}

impl ValueExecution for ConCat {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ValueResult {
        let words = &self
            .concat
            .iter()
            .map(|e| e.run_with_context(pico_rules, runtime, ctx))
            .filter(|x| x.is_ok())
            .filter_map(Result::ok)
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

        Ok(PicoValue::String(tt.join("")))
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

impl ValueExecution for Slice {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ValueResult {
        info!("slicing");
        let s = self.slice.0.run_with_context(pico_rules, runtime, ctx)?;
        let start_index = self.slice.1;
        //let endIndex = self.slice.2;

        match s {
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
                    return Ok(PicoValue::String(substring.to_string()));
                }

                return Ok(PicoValue::String("".to_string()));
            }
            _ => Err(PicoError::IncompatibleComparison(s, PicoValue::Null)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PointerValue {
    InputPointer(String),
    VarPointer(String, VarLookup),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pointer {
    pointer: PointerValue, // JSON pointer
}
impl fmt::Display for PointerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            PointerValue::InputPointer(c) => write!(f, "JSON Pointer {}", c),
            PointerValue::VarPointer(s, v) => write!(f, "JSON Pointer {}, {:?}", s, v,),
        }
    }
}

impl ValueExecution for Pointer {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ValueResult {
        info!("consulting pointer");
        match &self.pointer {
            PointerValue::InputPointer(json_pointer) => {
                if let Some(input_json) = &ctx.input_json {
                    if let Some(value) = input_json.pointer(&json_pointer) {
                        return Ok(value.clone());
                    }
                }
                Err(PicoError::NoSuchValue(json_pointer.to_string()))
            }
            PointerValue::VarPointer(json_pointer, var) => {
                let value = var.run_with_context(pico_rules, runtime, ctx)?;
                if let Some(c) = value.pointer(&json_pointer) {
                    return Ok(c.clone());
                }

                Err(PicoError::NoSuchValue(json_pointer.to_string()))
            }
        }
        /*
        if let Some(json) = &ctx.input_json {
            trace!("we have some json, checking pointer {}", self.pointer);
        }
        */
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LiteralString(String);

impl ValueExecution for LiteralString {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> ValueResult {
        info!("HIT a literal string {}", self.0);

        Ok(PicoValue::String(self.0.to_string()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LiteralI64(i64);

impl ValueExecution for LiteralI64 {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ValueResult {
        info!("HIT a literal number {}", self.0);

        Ok(json!(self.0))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableLookup {
    /// table name
    lookup: (String, String), // table, key
}

// lookup can use a table in this file (InternalTable)
// or an (ExternalFile)
impl ValueExecution for TableLookup {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> ValueResult {
        info!(
            "Lookup Dictionary {:?} -> {:?}",
            self.lookup.0, self.lookup.1
        );

        let value: Option<&PicoValue> = match pico_rules.get_table(&self.lookup.0) {
            None => None,
            Some(table_type) => match table_type {
                LookupType::InternalTable(internal_table) => {
                    Some(internal_table.lookup(&self.lookup.1))
                }

                LookupType::ExternalTable(external_table) => {
                    runtime.table_lookup(external_table, &self.lookup.1)
                }
            },
        };

        match value {
            Some(v) => Ok(v.clone()),
            None => Err(PicoError::NoSuchValue(format!(
                "No Such table {}",
                &self.lookup.0
            ))),
        }
    }
}
