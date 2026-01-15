use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};
use tracing::info;

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Kusanagi Agent Controller is healthy")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting Kusanagi server on port 8080");

    HttpServer::new(|| {
        App::new()
            .service(health_check)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
