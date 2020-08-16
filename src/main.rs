//extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;
extern crate tinytemplate;
extern crate valico;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

use anyhow::Result;
use clap::{App, Arg};

extern crate picolang;

use picolang::include::PicoRules;
use picolang::server::serve;
use picolang::values::PicoValue;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{reply::json, Filter, Rejection, Reply};

//type Result<T> = std::result::Result<T, Rejection>;

trait Initializable {
    fn init(&self) -> bool {
        return true;
    }
}

#[derive(Serialize, Debug)]
pub struct HealthResponse {
    ok: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let matches: clap::ArgMatches = App::new("Pico Lang")
        .version("0.1")
        .arg(
            Arg::new("rules")
                .long("rules")
                .default_value("file://pico-rule.json")
                .value_name("FILE")
                .takes_value(true),
        )
        .get_matches();
    info!("Matches {:?}", matches);

    if let Some(ref file) = matches.value_of("rules") {
        info!("filename {}", file);
    }

    debug!("Hello, world! ");

    let nc = nats::connect("localhost").unwrap();
    let sub = nc
        .subscribe("my.subject")
        .unwrap()
        .with_handler(move |msg| {
            println!("Received {}", &msg);
            Ok(())
        });
    nc.publish("my.subject", "Hello World!");

    let file = matches.value_of("rules").unwrap();
    let mut pr = PicoRules::new(file);
    let _x: picolang::commands::execution::ExecutionResult = pr.load().unwrap();
    let pico: Arc<RwLock<PicoRules>> = Arc::new(RwLock::new(pr));

    serve(pico).await;
    Ok(())
}
