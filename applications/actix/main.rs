use futures_core::stream::Stream;
use futures_util::StreamExt;
use std::future::Future;
use std::sync::Mutex;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

use picolang::rules::loaders::FileLoader;
use picolang::rules::PicoRules;
use picolang::runtime::{PicoRulesCache, PicoRuntime};

#[macro_use]
extern crate log;

async fn submit<'a>(
  data: web::Data<Mutex<PicoRuntime<'a>>>,
  mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
  // extract the full body
  let mut body = web::BytesMut::new();
  while let Some(chunk) = payload.next().await {
    body.extend_from_slice(&chunk?);
  }

  // then desrialize
  let obj = serde_json::from_slice::<serde_json::Value>(&body)?;

  let mut rt = data.lock().unwrap();
  let mut ctx = rt.make_ctx().set_json(obj);

  rt.exec_root_with_context(&mut ctx);
  rt.new_namespace("lop");

  let f = ctx.get_final_ctx();

  Ok(HttpResponse::Ok().json(f))
}

async fn rules<'a>(data: web::Data<Mutex<PicoRuntime<'a>>>) -> Result<HttpResponse, Error> {
  let rt = data.lock().unwrap();
  let rules = rt.rule_file_names();
  Ok(HttpResponse::Ok().json(rules))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

  // let cache_data = web::Data::new(cache);

  //let mut rt = PicoRuntime::new(nv, cache, "pico-rule.json").initialise();
  let mut rt = PicoRuntime::new().initialise();

  let data = web::Data::new(Mutex::new(rt));

  HttpServer::new(move || {
    App::new()
      .app_data(data.clone())
      .service(web::resource("/submit").route(web::post().to(submit)))
      .service(web::resource("/rules").route(web::get().to(rules)))
  })
  .workers(32)
  .bind("127.0.0.1:8000")?
  .run()
  .await
}
