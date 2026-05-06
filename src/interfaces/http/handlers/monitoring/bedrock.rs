use axum::{extract::State, response::IntoResponse};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use aws_credential_types::Credentials;

use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

pub async fn bedrock_metrics_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let region = std::env::var("BEDROCK_AWS_REGION")
        .or_else(|_| std::env::var("AWS_REGION"))
        .unwrap_or_else(|_| "us-east-1".to_string());

    let account_label = std::env::var("BEDROCK_ACCOUNT_LABEL")
        .unwrap_or_else(|_| "default".to_string());

    let budget_limit: f64 = std::env::var("BEDROCK_BUDGET_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0);

    match fetch_bedrock_all_metrics(&state, &region, budget_limit).await {
        Ok(metrics) => api_success(json!({
            "account_label": account_label,
            "region": region,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "bedrock": metrics,
        })),
        Err(e) => api_error(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            format!("Bedrock metrics unavailable: {}", e),
        ),
    }
}

async fn build_aws_config(region: &str) -> aws_config::SdkConfig {
    // Support dedicated Bedrock credentials separate from S3/MinIO credentials
    // Priority: BEDROCK_AWS_* > BEDROCK_MONITORING_ROLE_ARN > default AWS chain
    //
    // Use defaults(BehaviorVersion::latest()) instead of from_env() to ensure
    // proper connector/runtime initialization (compatible with default-features = false)
    let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_config::Region::new(region.to_string()));

    if let (Ok(key_id), Ok(secret)) = (
        std::env::var("BEDROCK_AWS_ACCESS_KEY_ID"),
        std::env::var("BEDROCK_AWS_SECRET_ACCESS_KEY"),
    ) {
        tracing::info!("Using dedicated Bedrock AWS credentials (BEDROCK_AWS_*)");
        let creds = Credentials::new(
                key_id,
                secret,
                None,
                None,
                "bedrock-dedicated",
            );
        loader = loader.credentials_provider(creds);
    } else if let Ok(role_arn) = std::env::var("BEDROCK_MONITORING_ROLE_ARN") {
        let mut builder = aws_config::sts::AssumeRoleProvider::builder(&role_arn)
            .session_name("kusanagi-bedrock-monitoring");

        if let Ok(ext_id) = std::env::var("BEDROCK_MONITORING_EXTERNAL_ID") {
            builder = builder.external_id(&ext_id);
        }

        loader = loader.credentials_provider(builder.build().await);
        tracing::info!("Using cross-account role: {}", role_arn);
    } else {
        tracing::warn!("No dedicated Bedrock credentials found, falling back to default AWS chain (may use MinIO keys)");
    }

    loader.load().await
}

async fn fetch_bedrock_all_metrics(
    _state: &AppState,
    region: &str,
    budget_limit: f64,
) -> Result<serde_json::Value, String> {
    let config = build_aws_config(region).await;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let cloudwatch = aws_sdk_cloudwatch::Client::new(&config);
    let costexplorer = aws_sdk_costexplorer::Client::new(&config);

    let mut result = serde_json::Map::new();

    let periods: Vec<(&str, i64, i64, i32)> = vec![
        ("last_24h", now - (24 * 3600), now, 3600),
        ("last_7d", now - (7 * 86400), now, 86400),
    ];

    use aws_sdk_cloudwatch::types::Statistic;

    for (period_label, start, end, period) in &periods {
        let mut period_data = serde_json::Map::new();

        for metric_name in ["InvocationCount", "InputTokenCount", "OutputTokenCount", "InvocationLatency"] {
            let stat: Statistic = if metric_name == "InvocationLatency" {
                Statistic::Average
            } else {
                Statistic::Sum
            };

            let response = cloudwatch
                .get_metric_statistics()
                .namespace("AWS/Bedrock")
                .metric_name(metric_name)
                .start_time(aws_sdk_cloudwatch::primitives::DateTime::from_secs(*start))
                .end_time(aws_sdk_cloudwatch::primitives::DateTime::from_secs(*end))
                .period(*period)
                .statistics(stat)
                .send()
                .await
                .map_err(|e| {
                    tracing::error!(?e, "CloudWatch query failed for {}", metric_name);
                    format!("CloudWatch query error for {}: {e:?}", metric_name)
                })?;

            let datapoints = response.datapoints();
            let total: f64 = if metric_name == "InvocationLatency" {
                datapoints.iter().filter_map(|dp| dp.average()).next_back().unwrap_or(0.0)
            } else {
                datapoints.iter().filter_map(|dp| dp.sum()).sum()
            };

            period_data.insert(metric_name.to_string(), json!({
                "total": total,
            }));
        }

        result.insert(period_label.to_string(), json!(period_data));
    }

    let cost_data = fetch_bedrock_cost(&costexplorer).await;
    result.insert("cost".to_string(), cost_data);

    if let Some(cost) = result.get("cost") {
        if let Some(total_cost) = cost.get("total_cost").and_then(|c| c.as_f64()) {
            let usage_pct = (total_cost / budget_limit) * 100.0;
            result.insert("budget".to_string(), json!({
                "limit": budget_limit,
                "actual_spend": total_cost,
                "usage_percent": usage_pct,
                "needs_regeneration": usage_pct > 80.0,
            }));
        }
    }

    Ok(json!(result))
}

async fn fetch_bedrock_cost(ce: &aws_sdk_costexplorer::Client) -> serde_json::Value {
    let now = chrono::Utc::now();
    let start = (now - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
    let end = now.format("%Y-%m-%d").to_string();

    let interval = match aws_sdk_costexplorer::types::DateInterval::builder()
        .start(&start)
        .end(&end)
        .build()
    {
        Ok(i) => i,
        Err(e) => {
            tracing::warn!("Failed to build DateInterval: {}", e);
            return json!({ "total_cost": 0.0, "daily_costs": [], "error": e.to_string() });
        }
    };

    let filter = aws_sdk_costexplorer::types::Expression::builder()
        .dimensions(
            aws_sdk_costexplorer::types::DimensionValues::builder()
                .key(aws_sdk_costexplorer::types::Dimension::Service)
                .values("Amazon Bedrock")
                .build(),
        )
        .build();

    let result = ce
        .get_cost_and_usage()
        .time_period(interval)
        .granularity(aws_sdk_costexplorer::types::Granularity::Daily)
        .metrics("UnblendedCost")
        .metrics("UsageQuantity")
        .filter(filter)
        .send()
        .await;

    match result {
        Ok(response) => {
            let results = response.results_by_time();
            let total_cost: f64 = results
                .iter()
                .filter_map(|r| {
                    r.total()
                        .and_then(|t| t.get("UnblendedCost"))
                        .and_then(|c| c.amount())
                        .and_then(|a| a.parse::<f64>().ok())
                })
                .sum();

            let mut daily_costs: Vec<serde_json::Value> = Vec::new();
            for r in results {
                if let Some(tp) = r.time_period() {
                    let date = tp.start();
                    let cost = r.total()
                        .and_then(|t| t.get("UnblendedCost"))
                        .and_then(|c| c.amount())
                        .and_then(|a| a.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    daily_costs.push(json!({ "date": date, "cost": cost }));
                }
            }

            json!({
                "total_cost": total_cost,
                "daily_costs": daily_costs,
            })
        }
        Err(e) => {
            tracing::warn!("Cost Explorer query failed: {}", e);
            json!({ "total_cost": 0.0, "daily_costs": [], "error": e.to_string() })
        }
    }
}
