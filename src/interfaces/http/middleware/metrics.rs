use axum::{extract::Request, middleware::Next, response::IntoResponse};
use metrics::Label;
use std::time::Instant;

pub async fn track_metrics(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = req.uri().path().to_owned();
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    metrics::histogram!(
        "http_requests_duration_seconds",
        "path" => path.clone(),
        "method" => method.to_string(),
        "status" => status.clone()
    ).record(latency);
    
    metrics::counter!(
        "http_requests_total",
        "path" => path,
        "method" => method.to_string(),
        "status" => status
    ).increment(1);

    response
}
