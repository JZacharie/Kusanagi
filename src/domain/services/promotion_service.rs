use serde_json::{json, Value};
use std::env;
use reqwest::Client;
use base64::{engine::general_purpose, Engine as _};

pub async fn promote_to_production() -> Result<Value, String> {
    let pat = env::var("GH_PAT").map_err(|_| "GH_PAT not set".to_string())?;
    let client = Client::builder()
        .user_agent("Kusanagi-Dashboard/0.3.0")
        .build()
        .map_err(|e| e.to_string())?;

    let repo = "JZacharie/jo3";
    let dev_values_path = "values/infrastructure/kusanagi/values-dev.yaml";
    let prod_values_path = "values/infrastructure/kusanagi/values.yaml";

    // 1. Fetch Dev Tag from values-dev.yaml
    let dev_url = format!("https://api.github.com/repos/{}/contents/{}", repo, dev_values_path);
    let dev_resp = client.get(&dev_url)
        .bearer_auth(&pat)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch dev values: {}", e))?;

    if !dev_resp.status().is_success() {
        return Err(format!("GitHub API error fetching dev values: {}", dev_resp.status()));
    }

    let dev_content_json: Value = dev_resp.json().await.map_err(|e| e.to_string())?;
    let dev_content_b64 = dev_content_json["content"].as_str().ok_or("No content in dev values response")?.replace("\n", "");
    let dev_content_bytes = general_purpose::STANDARD.decode(dev_content_b64).map_err(|e| e.to_string())?;
    let dev_content_str = String::from_utf8(dev_content_bytes).map_err(|e| e.to_string())?;

    // Basic extraction of tag from YAML (regex would be better but let's keep it simple for now)
    let dev_tag = dev_content_str.lines()
        .find(|line| line.trim().starts_with("tag:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|tag| tag.trim().trim_matches('"'))
        .ok_or("Could not find tag in values-dev.yaml")?;

    tracing::info!("Found dev tag: {}", dev_tag);

    // 2. Fetch Prod values.yaml to get SHA
    let prod_url = format!("https://api.github.com/repos/{}/contents/{}", repo, prod_values_path);
    let prod_resp = client.get(&prod_url)
        .bearer_auth(&pat)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch prod values: {}", e))?;

    if !prod_resp.status().is_success() {
        return Err(format!("GitHub API error fetching prod values: {}", prod_resp.status()));
    }

    let prod_content_json: Value = prod_resp.json().await.map_err(|e| e.to_string())?;
    let prod_sha = prod_content_json["sha"].as_str().ok_or("No SHA in prod values response")?;
    let prod_content_b64 = prod_content_json["content"].as_str().ok_or("No content in prod values response")?.replace("\n", "");
    let prod_content_bytes = general_purpose::STANDARD.decode(prod_content_b64).map_err(|e| e.to_string())?;
    let prod_content_str = String::from_utf8(prod_content_bytes).map_err(|e| e.to_string())?;

    // 3. Update prod content
    // Replace tag
    let mut new_content = String::new();
    let mut tag_updated = false;
    let mut features_checked = false;

    for line in prod_content_str.lines() {
        if line.trim().starts_with("tag:") && !tag_updated {
            new_content.push_str(&format!("  tag: \"{}\"\n", dev_tag));
            tag_updated = true;
        } else if line.trim().contains("KUSANAGI_FULL_FEATURES") {
            new_content.push_str(line);
            new_content.push('\n');
            features_checked = true;
        } else if features_checked && line.trim().starts_with("value:") {
            new_content.push_str("    value: \"false\"\n");
            features_checked = false;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // 4. Commit changes
    let commit_message = format!("🚀 Release: Promote kusanagi to prod (tag: {})", dev_tag);
    let update_payload = json!({
        "message": commit_message,
        "content": general_purpose::STANDARD.encode(new_content),
        "sha": prod_sha,
        "branch": "main"
    });

    let put_resp = client.put(&prod_url)
        .bearer_auth(&pat)
        .json(&update_payload)
        .send()
        .await
        .map_err(|e| format!("Failed to commit update: {}", e))?;

    if !put_resp.status().is_success() {
        let error_body = put_resp.text().await.unwrap_or_default();
        return Err(format!("GitHub API error on commit: {}", error_body));
    }

    Ok(json!({
        "success": true,
        "message": commit_message,
        "tag": dev_tag
    }))
}
