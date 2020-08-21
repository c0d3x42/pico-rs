use crate::context::PicoContext;
//use crate::include::PicoRules;
use crate::invar::InVars;
use crate::loader::{PicoRules, PicoRuntime};
use crate::values::PicoValue;
use std::convert::Infallible;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use warp::{reply::json, Filter, Rejection, Reply};

pub async fn serve(pico: Arc<RwLock<PicoRules>>) {
    let submit = warp::path("submit");
    let submit_route = submit
        .and(warp::post())
        .and(warp::body::json())
        .and(with_pico(pico.clone()))
        .and_then(submit_handler);

    let routes = submit_route.with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResponse {
    output: String,
}

pub async fn submit_handler(
    body: InVars,
    pico: Arc<RwLock<PicoRules>>,
) -> Result<impl Reply, Rejection> {
    let re = pico.read().await;
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

    let mut runtime = PicoRuntime::new();
    re.run_with_context(&mut runtime, &mut ctx);
    println!("\n FINAL FINAL CTX {:?}", ctx);

    Ok(json(&ctx))
    /*
    Ok(json(&SubmitResponse {
        output: "lop".to_string(),
    }))
    */
}

pub fn with_pico(
    pico: Arc<RwLock<PicoRules>>,
) -> impl Filter<Extract = (Arc<RwLock<PicoRules>>,), Error = Infallible> + Clone {
    warp::any().map(move || pico.clone())
}
