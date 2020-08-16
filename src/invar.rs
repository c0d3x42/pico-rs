use serde::Deserialize;
use serde_json::Value;

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct InVars {
    #[serde(flatten)]
    pub input_map: HashMap<String, Value>,
}
