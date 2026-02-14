use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

/// Setup Prometheus recorder
/// Use install_recorder to set it as the global recorder
pub fn setup_metrics() -> anyhow::Result<PrometheusHandle> {
    let builder = PrometheusBuilder::new();
    let handle = builder.install_recorder()?;
    Ok(handle)
}
