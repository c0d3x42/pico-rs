use clap::{App as ClApp, Arg};
use futures_util::StreamExt;
use std::sync::Mutex;

use actix_web::{post, web, App, Error, HttpResponse, HttpServer};

use picolang::rules::RuleFile;
use picolang::runtime::PicoRuntime;

#[macro_use]
extern crate log;

async fn exec_rule<'a>(
  rulename: &str,
  runtime: &PicoRuntime<'a>,
  mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
  if !runtime.has_rule(&rulename) {
    warn!("Rule does not exist {}", rulename);
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

  let mut ctx = runtime.make_ctx(json);

  match runtime.exec_rule_with_context(&rulename, &mut ctx) {
    Ok(final_ctx) => HttpResponse::Ok().json(final_ctx).await,
    Err(x) => {
      error!("rule failed {}", x);
      let s = format!("{}", x);
      let err = vec![s];
      HttpResponse::NotFound().json(err).await
    }
  }
}

#[post("{rulename}")]
async fn submit_with_rulename<'a>(
  rulename: web::Path<String>,
  data_rt: web::Data<Mutex<PicoRuntime<'a>>>,
  payload: web::Payload,
) -> Result<HttpResponse, Error> {
  let runtime = data_rt.lock().unwrap();
  exec_rule(&rulename, &runtime, payload).await
}

async fn submit_default<'a>(
  data_rt: web::Data<Mutex<PicoRuntime<'a>>>,
  payload: web::Payload,
) -> Result<HttpResponse, Error> {
  let runtime = data_rt.lock().unwrap();
  exec_rule(&runtime.get_default_rule(), &runtime, payload).await
}

async fn rules<'a>(data: web::Data<Mutex<PicoRuntime<'a>>>) -> Result<HttpResponse, Error> {
  let rt = data.lock().unwrap();
  let rules = rt.rule_file_names();
  Ok(HttpResponse::Ok().json(rules))
}

async fn get_rule_by_name<'a>(
  data: web::Data<Mutex<PicoRuntime<'a>>>,
  rulename: web::Path<String>,
) -> Result<HttpResponse, Error> {
  let rt = data.lock().unwrap();

  let maybe_rulefile = rt.get_rule(&rulename);

  match maybe_rulefile {
    Some(rulefile) => Ok(HttpResponse::Ok().json(rulefile)),
    None => HttpResponse::NotFound().await,
  }
}

async fn post_rule_by_name<'a>(
  data: web::Data<Mutex<PicoRuntime<'a>>>,
  rulename: web::Path<String>,
  rulefile: web::Json<RuleFile>,
) -> Result<HttpResponse, Error> {
  info!("GOT a rulefile {}", rulefile);

  let mut rt = data.lock().unwrap();
  rt.post_rule(&rulename, rulefile.into_inner());
  HttpResponse::Ok().await
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
        .default_value("pico.rule.json")
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
  let rt = PicoRuntime::new()
    .set_rules_directory(&rules_directory)
    .initialise()
    .set_default_rule(&entry_rule);

  let data = web::Data::new(Mutex::new(rt));

  HttpServer::new(move || {
    App::new()
      .app_data(data.clone())
      .service(web::resource("/submit").route(web::post().to(submit_default)))
      .service(web::scope("/submit/").service(submit_with_rulename))
      .service(web::resource("/rules").route(web::get().to(rules)))
      .route("/rule/{rulename}", web::get().to(get_rule_by_name))
      .route("/rule/{rulename}", web::post().to(post_rule_by_name))
  })
  .workers(32)
  .bind(binding_to)?
  .run()
  .await
}
