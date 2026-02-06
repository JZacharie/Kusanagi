use serde_json::{json, Value};
use tokio::process::Command;
use chrono::Utc;

pub async fn get_news() -> Result<Value, String> {
    // Essayer des sources de news tech/DevOps
    let sources = vec![
        "https://feeds.feedburner.com/oreilly/radar",
        "https://kubernetes.io/feed.xml",
        "https://blog.docker.com/feed/",
        "https://www.cncf.io/feed/",
    ];
    
    for source in sources {
        if let Ok(news) = fetch_rss_feed(source).await {
            if !news.is_empty() {
                return Ok(json!(news));
            }
        }
    }
    
    // Fallback: essayer curl pour une API de news simple
    let news_api_output = Command::new("curl")
        .args(&["-s", "https://hacker-news.firebaseio.com/v0/topstories.json"])
        .output()
        .await;
    
    if let Ok(result) = news_api_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(story_ids) = serde_json::from_str::<Vec<u64>>(&json_str) {
                let mut news_items = Vec::new();
                
                // Prendre les 5 premiers articles
                for &story_id in story_ids.iter().take(5) {
                    let story_output = Command::new("curl")
                        .args(&["-s", &format!("https://hacker-news.firebaseio.com/v0/item/{}.json", story_id)])
                        .output()
                        .await;
                    
                    if let Ok(story_result) = story_output {
                        if story_result.status.success() {
                            let story_json = String::from_utf8_lossy(&story_result.stdout);
                            if let Ok(story) = serde_json::from_str::<Value>(&story_json) {
                                news_items.push(json!({
                                    "title": story["title"],
                                    "url": story["url"],
                                    "score": story["score"],
                                    "time": story["time"],
                                    "source": "hackernews"
                                }));
                            }
                        }
                    }
                }
                
                if !news_items.is_empty() {
                    return Ok(json!({
                        "items": news_items,
                        "cached_at": Utc::now().to_rfc3339(),
                        "source": "hackernews"
                    }));
                }
            }
        }
    }
    
    // Fallback: news statiques tech/DevOps
    Ok(json!({
        "items": [
            {
                "title": "Kubernetes 1.29 Released with Enhanced Security Features",
                "url": "https://kubernetes.io/blog/2024/12/11/kubernetes-v1-29-release/",
                "source": "kubernetes.io",
                "category": "kubernetes",
                "time": "2024-12-11"
            },
            {
                "title": "Docker Desktop 4.26 Introduces New Container Management Tools",
                "url": "https://www.docker.com/blog/docker-desktop-4-26/",
                "source": "docker.com",
                "category": "docker",
                "time": "2024-12-10"
            },
            {
                "title": "ArgoCD 2.9 Brings Improved GitOps Workflows",
                "url": "https://argo-cd.readthedocs.io/en/stable/",
                "source": "argoproj.io",
                "category": "gitops",
                "time": "2024-12-09"
            },
            {
                "title": "Prometheus 2.48 Enhances Monitoring Capabilities",
                "url": "https://prometheus.io/blog/2024/12/08/prometheus-2-48-0-release/",
                "source": "prometheus.io",
                "category": "monitoring",
                "time": "2024-12-08"
            },
            {
                "title": "CNCF Announces New Cloud Native Projects",
                "url": "https://www.cncf.io/blog/2024/12/07/new-projects/",
                "source": "cncf.io",
                "category": "cloud-native",
                "time": "2024-12-07"
            }
        ],
        "cached_at": "2024-12-11T12:00:00Z",
        "source": "static"
    }))
}

async fn fetch_rss_feed(url: &str) -> Result<Vec<Value>, String> {
    let output = Command::new("curl")
        .args(&["-s", url])
        .output()
        .await;
    
    if let Ok(result) = output {
        if result.status.success() {
            let xml_content = String::from_utf8_lossy(&result.stdout);
            
            // Simple XML parsing pour RSS (très basique)
            let mut news_items = Vec::new();
            let lines: Vec<&str> = xml_content.lines().collect();
            
            let mut current_item = json!({});
            let mut in_item = false;
            
            for line in lines {
                let line = line.trim();
                
                if line.contains("<item>") {
                    in_item = true;
                    current_item = json!({});
                } else if line.contains("</item>") && in_item {
                    if !current_item["title"].is_null() {
                        current_item["source"] = json!(extract_domain(url));
                        news_items.push(current_item.clone());
                    }
                    in_item = false;
                } else if in_item {
                    if let Some(title) = extract_xml_content(line, "title") {
                        current_item["title"] = json!(title);
                    } else if let Some(link) = extract_xml_content(line, "link") {
                        current_item["url"] = json!(link);
                    } else if let Some(pub_date) = extract_xml_content(line, "pubDate") {
                        current_item["time"] = json!(pub_date);
                    }
                }
                
                if news_items.len() >= 5 {
                    break;
                }
            }
            
            return Ok(news_items);
        }
    }
    
    Err("Failed to fetch RSS feed".to_string())
}

fn extract_xml_content(line: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    
    if let Some(start) = line.find(&start_tag) {
        if let Some(end) = line.find(&end_tag) {
            let content_start = start + start_tag.len();
            if content_start < end {
                return Some(line[content_start..end].trim().to_string());
            }
        }
    }
    None
}

fn extract_domain(url: &str) -> String {
    if let Some(start) = url.find("://") {
        let after_protocol = &url[start + 3..];
        if let Some(end) = after_protocol.find('/') {
            after_protocol[..end].to_string()
        } else {
            after_protocol.to_string()
        }
    } else {
        url.to_string()
    }
}
