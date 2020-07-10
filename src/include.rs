use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::command::RuleFile;
use crate::command::{Execution, ExecutionResult, FnResult, RuleFileRoot};
use crate::context::{Context, PicoState};
use crate::values::PicoValue;

pub struct IncludedFileSeed<T = RuleFile> {
    lookups: HashMap<String, T>,
}

impl IncludedFileSeed {
    fn new() -> Self {
        IncludedFileSeed {
            lookups: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct IncludeFileDriver {
    filename: String,
}

impl<'de> Deserialize<'de> for IncludeFileDriver {
    //type Value = IncludeFileDriver;

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // use serde::de::Error;
        debug!("deserializing.1 ");
        let s = String::deserialize(deserializer)?;
        debug!("deserializing {:?}", s);

        //let nf: RuleFile = serde_json::from_reader(File::open(&s).unwrap()).unwrap();
        //debug!("NEW RULE FILE {:?}", nf);

        Ok(IncludeFileDriver { filename: s })
    }
}

impl Serialize for IncludeFileDriver {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // FIXME: needs to write out the self.rule to self._filename
        serializer.serialize_str(&self.filename)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludeFile {
    include: IncludeFileDriver,
}

impl Execution for IncludeFile {
    fn name(&self) -> String {
        format!("include [{:?}]", self.include).to_string()
    }

    fn run_with_context(&self, state: &mut PicoState, ctx: &mut Context) -> FnResult {
        info!("running included module");
        //self.include.rule.run_with_context(state, ctx)
        Ok(ExecutionResult::Continue(PicoValue::Boolean(true)))
    }
}

pub struct RequireSeed<T> {
    required: HashMap<String, T>,
}

impl<T> RequireSeed<T> {
    pub fn new() -> Self {
        RequireSeed {
            required: HashMap::new(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct RequireFile {
    require: String,
}

impl<'de> DeserializeSeed<'de> for RequireSeed<RequireFile> {
    type Value = RequireFile;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(RequireFile { require: s })
    }
}
/*
impl<'de> Visitor<'de> for RequireSeed<RequireFile> {
    type Value = RequireFile;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct required")
    }

    fn visit_str<A>(mut self, mut s: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        Ok(RequireFile {
            require: String::from("lop"),
        })
    }
}
*/

#[derive(Debug)]
pub struct Annotated<T> {
    pub value: T,
    pub path: Option<String>,
}

#[derive(Debug)]
pub struct Info {
    pub path: String,
    pub hm: HashMap<String, String>,
}

#[derive(Debug)]
pub struct InfoSeed<T>(pub Info, pub std::marker::PhantomData<T>);

impl<T> InfoSeed<T> {
    pub fn blah(&self) -> () {
        println!("HM {:?}", self.0);
    }
}

impl<'de, T> DeserializeSeed<'de> for InfoSeed<T>
where
    T: Deserialize<'de>,
{
    type Value = Annotated<T>;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0.hm.insert(String::from("k"), String::from("v"));
        //let path = self.0.as_ref().map(|info| info.path.to_owned());
        //let hm = self.0.as_ref().map(|info| info.hm.to_owned());

        Ok(Annotated {
            value: Deserialize::deserialize(deserializer)?,
            path: Some(String::from("blah")),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Event {
    pub event_id: String,
}

#[derive(Debug)]
pub enum LoadResult {
    Ok,
    NoSuchFile,
    ParseError,
    UnknownError,
}

#[derive(Debug)]
pub struct LoadedFile<T> {
    result: LoadResult,
    content: Option<T>,
}

pub type LoadedCache<T> = HashMap<String, LoadedFile<T>>;

pub fn load_file(filename: &str, cache: &mut LoadedCache<RuleFile>) {
    if cache.contains_key(filename) {
        warn!("circular filename: {}", filename);
        return;
    }

    let f = File::open(filename);
    if f.is_err() {
        cache.insert(
            String::from(filename),
            LoadedFile {
                result: LoadResult::NoSuchFile,
                content: None,
            },
        );
        warn!("failed to open file {}", filename);
        return;
    }

    let nf: RuleFile = serde_json::from_reader(f.unwrap()).unwrap();
    //let nf: RuleFile = serde_json::from_reader(File::open(filename).unwrap()).unwrap();

    let include_filenames: Vec<&String> = nf
        .root
        .iter()
        .filter_map(|v| match v {
            RuleFileRoot::IncludeFile(f) => Some(&f.include.filename),
            _ => None,
        })
        .collect();

    println!("values: {:?}", include_filenames);
    for ifilename in include_filenames {
        if ifilename != filename {
            warn!("attempt to include self again");
            load_file(ifilename, cache);
        }
    }

    let lf: LoadedFile<RuleFile> = LoadedFile {
        result: LoadResult::Ok,
        content: Some(nf),
    };

    cache.insert(String::from(filename), lf);
}
