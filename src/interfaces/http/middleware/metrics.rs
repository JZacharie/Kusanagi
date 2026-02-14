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

    let labels = vec![
        Label::new("path", path),
        Label::new("method", method.to_string()),
        Label::new("status", status),
    ];

    // metrics 0.21 supports passing reference to Vec<Label> if we implement IntoLabels?
    // Actually, IntoLabels is implemented for Vec<Label>, [Label; N], etc.
    // References might need cloning.
    metrics::histogram!("http_requests_duration_seconds", latency, labels.clone());
    metrics::counter!("http_requests_total", 1, labels);

    response
}
