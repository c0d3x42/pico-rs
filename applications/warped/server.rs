use crate::app::AppOptions;
use picolang::rules::PicoRules;
use picolang::runtime::PicoRuntime;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use warp::{reply::json, Filter, Rejection, Reply};

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResponse {
    output: String,
}

pub async fn serve2<'a>(pico: Arc<RwLock<PicoRuntime<'a>>>, app: AppOptions) {
    info!("Serve {:?}", app);

    let submit = warp::path("submit");
    let submit_route = submit
        .and(warp::post())
        .and(warp::body::json())
        .and(with_pico2(pico.clone()))
        .and_then(submit_handler2);

    let routes = submit_route.with(warp::cors().allow_any_origin());

    let port = app.port;

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await
}

pub async fn submit_handler2<'a>(
    body: serde_json::Value,
    pico: Arc<RwLock<PicoRuntime<'a>>>,
) -> Result<impl Reply, Rejection> {
    let rt = pico.read().await;
    trace!("InputVars... {:?}", body);
    let mut ctx = rt.make_ctx().set_json(body);
    trace!("INITIAL CTX = {:?}", ctx);

    rt.exec_root_with_context(&mut ctx);

    info!("\n FINAL CTX {:?}", ctx);
    info!("\n FINAL runtime globals {:?}", rt.globals);

    Ok(json(&ctx.get_final_ctx()))
}

pub fn with_pico2(
    pico: Arc<RwLock<PicoRuntime>>,
) -> impl Filter<Extract = (Arc<RwLock<PicoRuntime>>,), Error = Infallible> + Clone {
    warp::any().map(move || pico.clone())
}
