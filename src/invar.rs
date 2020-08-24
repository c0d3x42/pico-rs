use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct InVars {
    //#[serde(flatten)]
//pub input_map: HashMap<String, Value>,
}

impl InVars {
    pub fn hello(&self) {
        info!("hello world");
    }
}
