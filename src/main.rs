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

use picolang::rules::PicoRules;

#[cfg(feature = "srv_nats")]
use picolang::nats::start_nats;

#[cfg(not(feature = "srv_nats"))]
async fn start_nats() {}

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
            Arg::new("rules")
                .long("rules")
                .default_value("pico-rule.json")
                .value_name("FILE")
                .takes_value(true),
        )
        .get_matches();
    info!("Matches {:?}", matches);

    if let Some(ref file) = matches.value_of("rules") {
        info!("filename {}", file);
    }

    debug!("Hello, world! ");

    let nr = PicoRules::new()
        .load_rulefile("pico-rule.json")
        .load_includes();
    debug!("NR = {}", nr);

    //    let mut ctx = PicoContext::new();
    //    let mut runtime = PicoRuntime::new(&nr).initialise();
    //    runtime.exec_root_with_context(&mut ctx);
    //nr.run_with_context(&mut runtime, &mut ctx);

    //    warn!("RUNTIME {:?}", runtime);
    info!("DONE");
    start_nats();

    let file = matches.value_of("rules").unwrap();
    //let mut pr = PicoRules::new(file);
    //let _x: picolang::commands::execution::ExecutionResult = pr.load().unwrap();
    let pico: Arc<RwLock<PicoRules>> = Arc::new(RwLock::new(nr));

    serve(pico).await;
    Ok(())
}
