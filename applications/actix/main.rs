use std::sync::Mutex;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct Lop {
  lop: i8,
}

async fn index() -> impl Responder {
  let l = Lop { lop: 1 };
  HttpResponse::Ok().json(l)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  HttpServer::new(move || App::new().service(web::resource("/").route(web::get().to(index))))
    .bind("127.0.0.1:1337")?
    .run()
    .await
}
