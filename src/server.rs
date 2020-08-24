use crate::app::AppOptions;
use crate::context::PicoContext;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;
use std::convert::Infallible;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use jsonpath_lib as jsonpath;

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
    let re = pico.read().await;

    trace!("InVars... {:?}", body);
    let mut ctx = PicoContext::new().set_json(body);
    trace!("INITIAL CTX = {:?}", ctx);

    let mut runtime = PicoRuntime::new(&re);
    re.run_with_context(&mut runtime, &mut ctx);
    info!("\n FINAL CTX {:?}", ctx);
    info!("\n FINAL runtime globals {:?}", runtime.globals);

    Ok(json(&ctx))
}

pub fn with_pico(
    pico: Arc<RwLock<PicoRules>>,
) -> impl Filter<Extract = (Arc<RwLock<PicoRules>>,), Error = Infallible> + Clone {
    warp::any().map(move || pico.clone())
}
