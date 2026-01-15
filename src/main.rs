use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use actix_files::Files;
use tracing::info;

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Kusanagi Agent Controller is healthy")
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting Kusanagi server on port 8080");
    info!("Access the cyberpunk interface at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .service(health_check)
            .service(index)
            .service(Files::new("/static", "./static").show_files_listing())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
