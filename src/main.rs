//extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;
extern crate tinytemplate;
extern crate valico;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

use std::collections::HashMap;

//use anyhow::Result;
use clap::{App, Arg};

extern crate picolang;

use picolang::context::PicoContext;
use picolang::include::PicoRules;
use picolang::invar::InVars;
use picolang::values::PicoValue;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{reply::json, Filter, Rejection, Reply};

type Result<T> = std::result::Result<T, Rejection>;

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
    let matches = App::new("Pico Lang")
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

    let mut ctx = PicoContext::new();
    ctx.variables
        .insert("x".to_string(), PicoValue::String("xxxx".to_string()));
    ctx.variables
        .insert("q".to_string(), PicoValue::String("QQQQ".to_string()));
    ctx.variables.insert("n".to_string(), PicoValue::Number(42));
    ctx.variables
        .insert("op".to_string(), PicoValue::String("OP".to_string()));

    let mut sth: HashMap<String, String> = HashMap::new();
    sth.insert(String::from("a"), String::from("A"));

    let file = matches.value_of("rules").unwrap();
    let mut pr = PicoRules::new(file);
    let x = pr.load().unwrap();
    let pico: Arc<RwLock<PicoRules>> = Arc::new(RwLock::new(pr));

    /*
    if let Some(ref file) = matches.value_of("rules") {
      let mut pr = PicoRules::new(file);
      let x = pr.load().unwrap();
      pico = Arc::new(RwLock::new(pr));

          match x {
            Ok(y) => {
              info!("GOT y ");
              let mut ps = pr.make_state();
              pr.run_with_context(&mut ps, &mut ctx);
            }
            Err(e) => {
              warn!("OOPS {}", e);
            }
          }

      println!("\n FINAL FINAL CTX {:?}", ctx.local_variables);
    }
      */

    let register = warp::path("register");
    let register_route = register
        .and(warp::post())
        .and(warp::body::json())
        .and_then(register_handler);

    let submit = warp::path("submit");
    let submit_route = submit
        .and(warp::post())
        .and(warp::body::json())
        .and(with_pico(pico.clone()))
        .and_then(submit_handler);

    let health_route = warp::path!("health").and_then(health_handler);
    let routes = health_route
        .or(register_route)
        .or(submit_route)
        .with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
    Ok(())
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok(json(&HealthResponse { ok: 1 }))
    //Ok(StatusCode::OK)
}
#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    user_id: usize,
}

#[derive(Serialize, Debug)]
pub struct RegisterResponse {
    uuid: String,
}

pub async fn register_handler(body: RegisterRequest) -> Result<impl Reply> {
    let user_id = body.user_id;

    let uuid = Uuid::new_v4().to_string();

    Ok(json(&RegisterResponse { uuid }))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResponse {
    output: String,
}

pub async fn submit_handler(body: InVars, pico: Arc<RwLock<PicoRules>>) -> Result<impl Reply> {
    let mut re = pico.read().await;
    let mut state = re.make_state();

    let mut ctx = PicoContext::new();

    trace!("InVars... {:?}", body);

    for (key, value) in body.input_map {
        match value {
            serde_json::Value::String(s) => {
                ctx.variables.insert(key, PicoValue::String(s));
            }
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    let pv = match n.as_i64() {
                        Some(nn) => PicoValue::Number(nn),
                        None => PicoValue::Number(0),
                    };

                    ctx.variables.insert(key, pv);
                } else if n.is_u64() {
                    let pv = match n.as_u64() {
                        Some(nn) => PicoValue::UnsignedNumber(nn),
                        None => PicoValue::UnsignedNumber(0),
                    };

                    ctx.variables.insert(key, pv);
                }
            }
            _ => {
                warn!("Unsupported input var {}", key);
            }
        }
    }

    trace!("INITIAL CTX = {:?}", ctx);

    re.run_with_context(&mut state, &mut ctx);
    println!("\n FINAL FINAL CTX {:?}", ctx);

    Ok(json(&SubmitResponse {
        output: "lop".to_string(),
    }))
}

pub fn with_pico(
    pico: Arc<RwLock<PicoRules>>,
) -> impl Filter<Extract = (Arc<RwLock<PicoRules>>,), Error = Infallible> + Clone {
    warp::any().map(move || pico.clone())
}
