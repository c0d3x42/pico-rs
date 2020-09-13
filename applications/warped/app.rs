#[derive(Debug, Clone)]
pub struct AppOptions {
  pub rulefile: String,
  pub port: u32,
}

impl AppOptions {
  pub fn new() -> Self {
    Self {
      rulefile: "pico-rule.json".to_string(),
      port: 8000,
    }
  }
}
