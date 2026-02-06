use serde_json::{json, Value};
use chrono::Utc;
use reqwest::Client;

pub async fn get_news() -> Result<Value, String> {
    let client = Client::new();
    
    // Tech/DevOps sources
    let sources = vec![
        "https://feeds.feedburner.com/oreilly/radar",
        "https://kubernetes.io/feed.xml",
        "https://blog.docker.com/feed/",
        "https://www.cncf.io/feed/",
    ];
    
    // Try RSS feeds first
    for source in sources {
        if let Ok(news) = fetch_rss_feed(&client, source).await {
            if !news.is_empty() {
                return Ok(json!({
                    "items": news,
                    "cached_at": Utc::now().to_rfc3339(),
                    "source": "rss"
                }));
            }
        }
    }
    
    // Fallback: Hacker News API via reqwest
    if let Ok(response) = client.get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send().await 
    {
        if let Ok(story_ids) = response.json::<Vec<u64>>().await {
            let mut news_items = Vec::new();
            
            // Take top 5
            for &story_id in story_ids.iter().take(5) {
                if let Ok(story_res) = client.get(&format!("https://hacker-news.firebaseio.com/v0/item/{}.json", story_id))
                    .send().await 
                {
                    if let Ok(story) = story_res.json::<Value>().await {
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
            
            if !news_items.is_empty() {
                return Ok(json!({
                    "items": news_items,
                    "cached_at": Utc::now().to_rfc3339(),
                    "source": "hackernews"
                }));
            }
        }
    }
    
    // Fallback: Static news
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
            }
        ],
        "cached_at": Utc::now().to_rfc3339(),
        "source": "static"
    }))
}

async fn fetch_rss_feed(client: &Client, url: &str) -> Result<Vec<Value>, String> {
    let response = client.get(url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let xml_content = response.text().await.map_err(|e| e.to_string())?;
        
        let mut news_items = Vec::new();
        let lines: Vec<&str> = xml_content.lines().collect();
        
        let mut current_item = json!({});
        let mut in_item = false;
        
        for line in lines {
            let line = line.trim();
            
            if line.contains("<item>") || line.contains("<entry>") {
                in_item = true;
                current_item = json!({});
            } else if (line.contains("</item>") || line.contains("</entry>")) && in_item {
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
