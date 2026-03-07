use sqlx::postgres::PgPoolOptions;
use serde_json::{json, Value};
use std::env;

pub struct SteampipeStats {
    pub passed: i64,
    pub failed: i64,
    pub alarm: i64,
    pub info: i64,
    pub skip: i64,
}

pub async fn get_compliance_stats() -> Result<SteampipeStats, String> {
    let database_url = env::var("STEAMPIPE_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://steampipe:1pjVZE4bYBkIGWpNTOgl@steampipe.steampipe-powerpipe.svc:9193/steampipe".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| format!("Failed to connect to Steampipe DB: {}", e))?;

    // Query for kubernetes insights/compliance if possible
    // This is a generic query that works with powerpipe/steampipe if the tables exist
    let query = "
        select 
            status,
            count(*) as count
        from 
            kubernetes_compliance_result
        group by 
            status;
    ";

    let rows = sqlx::query(query)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    let mut stats = SteampipeStats {
        passed: 0,
        failed: 0,
        alarm: 0,
        info: 0,
        skip: 0,
    };

    for row in rows {
        use sqlx::Row;
        let status: String = row.get("status");
        let count: i64 = row.get("count");

        match status.to_lowercase().as_str() {
            "passed" | "ok" => stats.passed += count,
            "failed" | "error" => stats.failed += count,
            "alarm" => stats.alarm += count,
            "info" => stats.info += count,
            "skip" => stats.skip += count,
            _ => {}
        }
    }

    Ok(stats)
}

pub async fn get_security_score_metrics() -> Result<Value, String> {
    match get_compliance_stats().await {
        Ok(stats) => {
            let total = stats.passed + stats.failed + stats.alarm;
            let score = if total > 0 {
                (stats.passed as f64 / total as f64) * 100.0
            } else {
                100.0 // Default to 100 if no checks found
            };

            Ok(json!({
                "score": score,
                "passed": stats.passed,
                "failed": stats.failed,
                "alarm": stats.alarm,
                "info": stats.info,
                "skip": stats.skip,
                "total_checks": total
            }))
        },
        Err(e) => {
            tracing::warn!("Steampipe query failed: {}. Using fallback score.", e);
            Ok(json!({
                "score": 85.0, // Fallback
                "note": "Steampipe data unavailable"
            }))
        }
    }
}
