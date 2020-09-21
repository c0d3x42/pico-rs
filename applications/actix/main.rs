use clap::{App as ClApp, Arg};
use futures_util::StreamExt;
use std::sync::Mutex;

use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

use picolang::errors::RuntimeError;
use picolang::runtime::PicoRuntime;

#[macro_use]
extern crate log;

#[post("/{rule}")]
async fn submit<'a>(
  rule: web::Path<(String)>,
  data_rt: web::Data<Mutex<PicoRuntime<'a>>>,
  mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
  info!("RULE {}", rule);
  let rt = data_rt.lock().unwrap();
  if !rt.has_rule(&rule) {
    warn!("Rule does not exist {}", rule);
    return HttpResponse::NotFound().await;
  }

  // extract the full body
  let mut body = web::BytesMut::new();
  while let Some(chunk) = payload.next().await {
    body.extend_from_slice(&chunk?);
  }

  // then deserialize
  let json_result = serde_json::from_slice::<serde_json::Value>(&body);

  // either get the parsed json or return BadRequest
  let json = json_result.map_err(|x| {
    error!("parse failure: {}", x);
    HttpResponse::BadRequest()
  })?;

  let mut ctx = rt.make_ctx(json);

  match rt.exec_rule_with_context(&rule, &mut ctx) {
    Ok(final_ctx) => Ok(HttpResponse::Ok().json(final_ctx)),
    Err(x) => {
      error!("rule failed {}", x);
      let s = format!("{}", x);
      let err = vec![s];
      HttpResponse::NotFound().json(err).await
    }
  }
}

async fn rules<'a>(data: web::Data<Mutex<PicoRuntime<'a>>>) -> Result<HttpResponse, Error> {
  let rt = data.lock().unwrap();
  let rules = rt.rule_file_names();
  Ok(HttpResponse::Ok().json(rules))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

  let app = ClApp::new("pico-rs-actix")
    .version("0.1")
    .arg(
      Arg::with_name("rules_dir")
        .long("rules")
        .default_value("rules")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("entry")
        .default_value("pico-rule.json")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("hostip")
        .long("hostip")
        .default_value("localhost")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("port")
        .long("port")
        .default_value("8000")
        .takes_value(true),
    );

  let matches = app.get_matches();

  let port: String = matches.value_of("port").unwrap_or("8000").parse().unwrap();
  let hostip: String = matches
    .value_of("hostip")
    .unwrap_or("localhost")
    .to_string();
  let binding_to: String = format!("{}:{}", hostip, port);
  info!("BINDING {}", binding_to);

  let entry_rule: String = matches.value_of("entry").unwrap_or_default().to_string();
  let rules_directory: String = matches
    .value_of("rules_dir")
    .unwrap_or_default()
    .to_string();

  // Create the Pico rules runtime using command line args
  let mut rt = PicoRuntime::new()
    .set_rules_directory(&rules_directory)
    .initialise()
    .set_default_rule(&entry_rule);

  let data = web::Data::new(Mutex::new(rt));

  HttpServer::new(move || {
    App::new()
      .app_data(data.clone())
      .service(web::scope("/submit").service(submit))
      //.service(web::resource("/submit").route(web::post().to(submit)))
      .service(web::resource("/rules").route(web::get().to(rules)))
  })
  .workers(32)
  .bind(binding_to)?
  .run()
  .await
}
