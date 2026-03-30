use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;
use native_tls::TlsConnector;

pub fn configure_insecure_s3(builder: S3ConfigBuilder) -> S3ConfigBuilder {
    tracing::warn!("⚠️  Configuring S3 client to IGNORE SSL certificate verification (insecure) via native-tls for hyper 0.14 bridge");
    
    let tls_connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build insecure tls connector");
        
    let mut http_connector = hyper_014::client::HttpConnector::new();
    http_connector.enforce_http(false);
    
    let connector = hyper_tls_05::HttpsConnector::from((
        http_connector,
        tls_connector.into()
    ));

    let http_client = HyperClientBuilder::new().build(connector);
    
    builder.http_client(http_client)
}
