use std::collections::HashMap;

pub type PicoHashMap = HashMap<String, String>;

#[derive(Debug)]
pub struct PicoContext {
  //pub values: HashMap<String,String>
  pub values: PicoHashMap,
}
impl PicoContext {
  pub fn new() -> Self {
    let mut t = PicoHashMap::new();
    t.insert("lop".to_string(), "LOP".to_string());
    info!("New PicoContext");
    Self { values: t }
  }

  pub fn get(&self, name: &str) -> Option<&String> {
    return self.values.get(name);
  }

  pub fn put(&mut self, key: &str, value: &str) {
    self.values.insert(key.to_string(), value.to_string());
  }
}
