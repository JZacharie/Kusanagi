use actix_web::{get, HttpResponse, Responder};
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram, Counter, Histogram};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: Counter = register_counter!(
        "kusanagi_http_requests_total",
        "Total number of HTTP requests received"
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: Histogram = register_histogram!(
        "kusanagi_http_request_duration_seconds",
        "HTTP request duration in seconds"
    ).unwrap();
}

#[get("/metrics")]
pub async fn metrics_handler() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!("Failed to encode metrics: {}", e);
        return HttpResponse::InternalServerError().body("Failed to encode metrics");
    }

    match String::from_utf8(buffer) {
        Ok(s) => HttpResponse::Ok().content_type("text/plain").body(s),
        Err(e) => {
            tracing::error!("Failed to convert metrics to string: {}", e);
            HttpResponse::InternalServerError().body("Failed to convert metrics to string")
        }
    }
}
