use futures_core::stream::Stream;
use futures_util::StreamExt;
use std::future::Future;
use std::sync::Mutex;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

use picolang::rules::loaders::FileLoader;
use picolang::rules::PicoRules;
use picolang::runtime::PicoRuntime;

#[macro_use]
extern crate log;

#[derive(Serialize)]
struct Lop {
  lop: i8,
}

#[derive(Deserialize, Serialize, Debug)]
struct Anything {
  xp: String,
}

async fn index() -> impl Responder {
  let l = Lop { lop: 1 };
  HttpResponse::Ok().json(l)
}

async fn pico2(
  data: web::Data<Mutex<PicoRuntime>>,
  mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
  let mut body = web::BytesMut::new();
  while let Some(chunk) = payload.next().await {
    body.extend_from_slice(&chunk?);
  }

  let obj = serde_json::from_slice::<serde_json::Value>(&body)?;

  let mut rt = data.lock().unwrap();
  let mut ctx = rt.make_ctx().set_json(obj);

  rt.exec_root_with_context(&mut ctx);

  let f = ctx.get_final_ctx();

  Ok(HttpResponse::Ok().json(f))
}

async fn pico(data: web::Data<Mutex<PicoRuntime>>, input: web::Json<Anything>) -> impl Responder {
  let mut rt = data.lock().unwrap();

  info!("RT = {:?}", rt);
  info!("input = {:?}", input);

  let mut ctx = rt.make_ctx();
  info!("CTX = {:?}", ctx);

  //rt.exec_root_with_context(&mut ctx);

  HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

  let fl = FileLoader::new("pico-rule.json");
  let nv = PicoRules::new().load_rulefile(fl);

  let mut rt = PicoRuntime::new(nv).initialise();

  let data = web::Data::new(Mutex::new(rt));

  HttpServer::new(move || {
    App::new()
      .app_data(data.clone())
      .service(web::resource("/").route(web::get().to(index)))
      .service(web::resource("/pico").route(web::post().to(pico)))
      .service(web::resource("/pico2").route(web::post().to(pico2)))
  })
  .bind("127.0.0.1:1337")?
  .run()
  .await
}
