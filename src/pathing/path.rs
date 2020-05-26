extern crate jsonpath_lib as jsonpath;
extern crate serde_json;
extern crate serde;



use serde_json::{Value };
use serde::{Serialize, Serializer, Deserialize};

#[derive(Debug)]
struct PathLookupInternal {
  template: String
}
impl PathLookupInternal{
  fn default()->PathLookupInternal{
    PathLookupInternal{ template: "lop".to_string()}
  }
}


#[derive(Debug,Deserialize)]
pub struct PathLookup {
    path: String,

    #[serde(skip, default="PathLookup::internal")]
    _internal: PathLookupInternal
}
impl PathLookup {

  fn internal()->PathLookupInternal{
    return PathLookupInternal::default();
  }
}

impl serde::Serialize for PathLookup{
  fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: Serializer,
  {
    PathLookupAugmented::from(self).serialize(s)
  }
}

#[derive(Debug, Serialize )]
pub struct PathLookupAugmented {
  path: String,

  template: String
}

impl<'a> From<&'a PathLookup> for PathLookupAugmented{
  fn from(other: &'a PathLookup) -> Self {
    let mut template = jsonpath::compile(&other.path);
    Self {
      path: other.path.clone(),
      template: "lop".to_string()
    }
  }
}
