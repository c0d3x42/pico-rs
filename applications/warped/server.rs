use crate::app::AppOptions;
use picolang::rules::PicoRules;
use picolang::runtime::PicoRuntime;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use warp::{reply::json, Filter, Rejection, Reply};

pub async fn serve(pico: Arc<RwLock<PicoRules>>, app: AppOptions) {
    info!("Serve {:?}", app);

    let submit = warp::path("submit");
    let submit_route = submit
        .and(warp::post())
        .and(warp::body::json())
        .and(with_pico(pico.clone()))
        .and_then(submit_handler);

    let routes = submit_route.with(warp::cors().allow_any_origin());

    let port = app.port;

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResponse {
    output: String,
}

pub async fn submit_handler(
    body: serde_json::Value,
    pico: Arc<RwLock<PicoRules>>,
) -> Result<impl Reply, Rejection> {
    let pico_rule = pico.read().await;

    /*
    let mut runtime = pico_rule.make_runtime();

    trace!("InputVars... {:?}", body);
    let mut ctx = runtime.make_ctx().set_json(body);
    trace!("INITIAL CTX = {:?}", ctx);

    runtime.exec_root_with_context(&mut ctx);

    info!("\n FINAL CTX {:?}", ctx);
    info!("\n FINAL runtime globals {:?}", runtime.globals);

    Ok(json(&ctx.get_final_ctx()))
    */
    Ok("xxx")
}

pub fn with_pico(
    pico: Arc<RwLock<PicoRules>>,
) -> impl Filter<Extract = (Arc<RwLock<PicoRules>>,), Error = Infallible> + Clone {
    warp::any().map(move || pico.clone())
}