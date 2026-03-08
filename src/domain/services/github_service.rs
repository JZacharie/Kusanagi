use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GithubWorkflowRun {
    pub id: u64,
    pub name: Option<String>,
    pub status: Option<String>,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub repository: Option<GithubRepoSummary>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GithubRepoSummary {
    pub name: String,
}

pub async fn get_last_pipelines() -> Result<Value, String> {
    let repositories = vec!["helmscharts", "Kusanagi"];
    let client = Client::builder()
        .user_agent("Kusanagi-Dashboard/0.3.0")
        .build()
        .map_err(|e| e.to_string())?;

    let mut all_runs = Vec::new();
    let pat = env::var("GH_PAT").ok();

    for repo in repositories {
        let url = format!("https://api.github.com/repos/JZacharie/{}/actions/runs?per_page=5", repo);
        let mut request = client.get(&url);
        
        if let Some(token) = &pat {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.map_err(|e| e.to_string())?;
        
        if response.status().is_success() {
            let data: Value = response.json().await.map_err(|e| e.to_string())?;
            if let Some(runs) = data["workflow_runs"].as_array() {
                for run in runs {
                    all_runs.push(json!({
                        "id": run["id"],
                        "repo": repo,
                        "name": run["name"],
                        "status": run["status"],
                        "conclusion": run["conclusion"],
                        "url": run["html_url"],
                        "created_at": run["created_at"],
                    }));
                }
            }
        }
    }

    // Sort by created_at descending across both repos
    all_runs.sort_by(|a, b| {
        b["created_at"].as_str().unwrap_or("").cmp(a["created_at"].as_str().unwrap_or(""))
    });

    Ok(json!(all_runs))
}
