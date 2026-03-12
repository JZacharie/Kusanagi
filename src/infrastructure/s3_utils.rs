use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;
use rustls_021::client::{ServerCertVerified, ServerCertVerifier};
use rustls_021::{Certificate, Error, ServerName};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug)]
struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}

pub fn configure_insecure_s3(builder: S3ConfigBuilder) -> S3ConfigBuilder {
    tracing::warn!("⚠️  Configuring S3 client to IGNORE SSL certificate verification");
    
    let rustls_config = rustls_021::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth();

    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(rustls_config)
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();

    let http_client = HyperClientBuilder::new().build(connector);
    
    builder.http_client(http_client)
}
