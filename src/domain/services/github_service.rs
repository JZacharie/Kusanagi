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
    let mut repo_stats = serde_json::Map::new();

    let now = chrono::Utc::now();
    let ten_days_ago = now - chrono::Duration::days(10);

    for repo in repositories {
        // 1. Fetch Pipeline Runs (more per page to have history)
        let runs_url = format!("https://api.github.com/repos/JZacharie/{}/actions/runs?per_page=30", repo);
        let mut runs_request = client.get(&runs_url);
        
        // 2. Fetch Pull Requests (open ones)
        let prs_url = format!("https://api.github.com/repos/JZacharie/{}/pulls?state=open", repo);
        let mut prs_request = client.get(&prs_url);

        if let Some(token) = &pat {
            runs_request = runs_request.bearer_auth(token);
            prs_request = prs_request.bearer_auth(token);
        }

        // Fetch both in parallel
        let (runs_response, prs_response) = tokio::join!(runs_request.send(), prs_request.send());

        // Process Runs
        if let Ok(response) = runs_response {
            if response.status().is_success() {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(runs) = data["workflow_runs"].as_array() {
                        tracing::info!("Fetched {} runs for repo {}", runs.len(), repo);
                        let mut count = 0;
                        for run in runs {
                            let created_at_str = run["created_at"].as_str().unwrap_or("");
                            if let Ok(created_at) = chrono::DateTime::parse_from_rfc3339(created_at_str) {
                                if created_at.with_timezone(&chrono::Utc) >= ten_days_ago {
                                    all_runs.push(json!({
                                        "id": run["id"],
                                        "repo": repo,
                                        "name": run["name"],
                                        "status": run["status"],
                                        "conclusion": run["conclusion"],
                                        "url": run["html_url"],
                                        "created_at": created_at_str,
                                    }));
                                    count += 1;
                                }
                            }
                        }
                        tracing::info!("Kept {} runs for repo {} after 10-day filter", count, repo);
                    }
                }
            } else {
                tracing::error!("Failed to fetch runs for repo {}: {}", repo, response.status());
            }
        }

        // Process PRs
        let pr_count = if let Ok(response) = prs_response {
            if response.status().is_success() {
                if let Ok(data) = response.json::<Value>().await {
                    data.as_array().map(|a| a.len()).unwrap_or(0)
                } else { 0 }
            } else { 0 }
        } else { 0 };

        repo_stats.insert(repo.to_string(), json!({
            "open_prs": pr_count,
            "prs_url": format!("https://github.com/JZacharie/{}/pulls", repo)
        }));
    }

    // Sort all runs by created_at descending
    all_runs.sort_by(|a, b| {
        b["created_at"].as_str().unwrap_or("").cmp(a["created_at"].as_str().unwrap_or(""))
    });

    Ok(json!({
        "pipelines": all_runs,
        "repo_stats": repo_stats
    }))
}
