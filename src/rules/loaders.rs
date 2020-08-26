use crate::commands::execution::AnyResult;
use crate::errors::RuleFileError;
use crate::rules::RuleFile;
use serde_json;
use std::fs::File;

pub type LoaderResult = AnyResult<RuleFile, RuleFileError>;

pub trait PicoLoader {
  //type RuleFileProvider;
  fn load(&self) -> LoaderResult;
  fn filename_is(&self) -> String;
}

pub struct FileLoader {
  filename: String,
}
impl FileLoader {
  pub fn new(filename: &str) -> Self {
    warn!("creating a new FileLoader");
    Self {
      filename: filename.to_string(),
    }
  }
}

impl PicoLoader for FileLoader {
  //type RuleFileProvider = FileLoader;

  fn filename_is(&self) -> String {
    self.filename.to_string()
  }

  fn load(&self) -> LoaderResult {
    info!("Loading... {}", self.filename);
    match File::open(&self.filename) {
      Ok(opened_file) => {
        info!("serde_json::from_reader...");
        let rule_file: RuleFile = serde_json::from_reader(opened_file).unwrap();
        Ok(rule_file)
      }
      Err(x) => {
        error!("failed to open: {:?}", x);
        Err(RuleFileError::ReadError {
          source: x,
          filename: self.filename.to_string(),
        })
      }
    }
  }
}
