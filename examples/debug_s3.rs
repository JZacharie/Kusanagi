use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client as S3Client;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env if exists
    dotenvy::dotenv().ok();

    let endpoint = env::var("S3_ENDPOINT")?;
    let access_key = env::var("S3_ACCESS_KEY")?;
    let secret_key = env::var("S3_SECRET_KEY")?;
    let bucket = env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi-news".to_string());
    let region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());

    println!("🛠️ Testing S3 connection to: {}", endpoint);
    println!("📦 Bucket: {}", bucket);
    println!("🌍 Region: {}", region);

    let credentials = Credentials::new(
        access_key,
        secret_key,
        None,
        None,
        "custom",
    );

    // Using the exact same builder pattern as the app
    let s3_config = aws_sdk_s3::config::Builder::new()
        .region(aws_sdk_s3::config::Region::new(region))
        .endpoint_url(&endpoint)
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();

    let client = S3Client::from_conf(s3_config);

    println!("🔍 Listing objects in bucket '{}'...", bucket);
    match client.list_objects_v2().bucket(&bucket).max_keys(5).send().await {
        Ok(output) => {
            println!("✅ Successfully connected to S3!");
            println!("📄 Found {} objects (max 5 shown)", output.contents().len());
            for obj in output.contents() {
                println!("  - {}", obj.key().unwrap_or("unknown"));
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to S3: {:?}", e);
            
            // Additional check: can we even curl it?
            println!("\n🔍 Attempting more raw diagnostics...");
            let http_client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true) // Just to see if it's a cert name issue vs cert trust issue
                .build()?;
            
            match http_client.get(&endpoint).send().await {
                Ok(resp) => println!("✅ Raw HTTP GET to endpoint worked (status: {})", resp.status()),
                Err(err) => eprintln!("❌ Raw HTTP GET failed: {:?}", err),
            }
            
            return Err(e.into());
        }
    }

    Ok(())
}
