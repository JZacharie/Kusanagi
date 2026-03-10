use serde_json::{json, Value};
use std::env;
use reqwest::Client;

pub async fn promote_to_production() -> Result<Value, String> {
    let pat = env::var("GH_PAT").map_err(|_| "GH_PAT not set".to_string())?;
    let client = Client::builder()
        .user_agent("Kusanagi-Dashboard/0.3.0")
        .build()
        .map_err(|e| e.to_string())?;

    let repo = "JZacharie/Kusanagi";
    let workflow_id = "promote.yml";

    // Trigger workflow_dispatch for promote.yml
    let url = format!("https://api.github.com/repos/{}/actions/workflows/{}/dispatches", repo, workflow_id);
    
    let payload = json!({
        "ref": "main"
    });

    let resp = client.post(&url)
        .bearer_auth(&pat)
        .header("Accept", "application/vnd.github.v3+json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to trigger promotion workflow: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let error_body = resp.text().await.unwrap_or_default();
        return Err(format!("GitHub API error ({}): {}", status, error_body));
    }

    Ok(json!({
        "success": true,
        "message": "Promotion workflow (promote.yml) triggered successfully on branch main."
    }))
}
