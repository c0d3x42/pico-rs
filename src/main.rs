//extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;
extern crate tinytemplate;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

use anyhow::Result;
use clap::{App, Arg};

extern crate picolang;

use picolang::rules::PicoRules;

#[cfg(feature = "srv_nats")]
use picolang::nats::start_nats;

#[cfg(not(feature = "srv_nats"))]
async fn start_nats() {}

use picolang::app::AppOptions;
use picolang::server::serve;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

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
            Arg::with_name("rules")
                .long("rules")
                .default_value("pico-rule.json")
                .value_name("FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .default_value("8000")
                .takes_value(true),
        )
        .get_matches();
    info!("Matches {:?}", matches);

    let mut app_options = AppOptions::new();

    app_options.rulefile = matches
        .value_of("rules")
        .unwrap_or("pico-rule.json")
        .to_string();

    app_options.port = matches.value_of("port").unwrap_or("8000").parse().unwrap();

    // for posterity
    debug!("Hello, world! ");

    let nr = PicoRules::new()
        .load_rulefile(&app_options.rulefile)
        .load_includes();
    trace!("NR = {}", nr);

    start_nats();

    let pico: Arc<RwLock<PicoRules>> = Arc::new(RwLock::new(nr));

    serve(pico, app_options).await;
    Ok(())
}
