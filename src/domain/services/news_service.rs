use serde_json::{json, Value};
use chrono::Utc;
use reqwest::Client;

pub async fn get_news() -> Result<Value, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    
    let mut all_news = Vec::new();
    
    // Fetch from all sources concurrently
    let (hn_news, korben_news, github_news, cncf_news) = tokio::join!(
        fetch_hackernews(&client),
        fetch_korben(&client),
        fetch_github_trending(&client),
        fetch_cncf(&client)
    );
    
    // Aggregate results
    if let Ok(mut items) = hn_news {
        all_news.append(&mut items);
    }
    if let Ok(mut items) = korben_news {
        all_news.append(&mut items);
    }
    if let Ok(mut items) = github_news {
        all_news.append(&mut items);
    }
    if let Ok(mut items) = cncf_news {
        all_news.append(&mut items);
    }
    
    if all_news.is_empty() {
        return Ok(json!({
            "items": get_fallback_news(),
            "cached_at": Utc::now().to_rfc3339(),
            "source": "fallback"
        }));
    }
    
    Ok(json!({
        "items": all_news,
        "cached_at": Utc::now().to_rfc3339(),
        "sources": ["hackernews", "korben", "github", "cncf"]
    }))
}

async fn fetch_hackernews(client: &Client) -> Result<Vec<Value>, String> {
    let response = client.get("https://hacker-news.firebaseio.com/v0/topstories.json")
        .send().await
        .map_err(|e| e.to_string())?;
    
    let story_ids = response.json::<Vec<u64>>().await
        .map_err(|e| e.to_string())?;
    
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
                    "source": "hackernews",
                    "icon": "🟠"
                }));
            }
        }
    }
    
    Ok(news_items)
}

async fn fetch_korben(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://korben.info/feed", "korben", "🔵").await
}

async fn fetch_github_trending(client: &Client) -> Result<Vec<Value>, String> {
    // GitHub trending doesn't have an official RSS, using a community service
    fetch_rss_feed(client, "https://mshibanami.github.io/GitHubTrendingRSS/daily/all.xml", "github", "🟣").await
}

async fn fetch_cncf(client: &Client) -> Result<Vec<Value>, String> {
    fetch_rss_feed(client, "https://www.cncf.io/feed/", "cncf", "📰").await
}

async fn fetch_rss_feed(client: &Client, url: &str, source_name: &str, icon: &str) -> Result<Vec<Value>, String> {
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

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
                current_item["source"] = json!(source_name);
                current_item["icon"] = json!(icon);
                news_items.push(current_item.clone());
            }
            in_item = false;
        } else if in_item {
            if let Some(title) = extract_xml_content(line, "title") {
                current_item["title"] = json!(clean_html(&title));
            } else if let Some(link) = extract_xml_content(line, "link") {
                current_item["url"] = json!(link);
            } else if line.contains("<link") && line.contains("href=") {
                // Atom feed link format
                if let Some(href_start) = line.find("href=\"") {
                    let after_href = &line[href_start + 6..];
                    if let Some(href_end) = after_href.find('"') {
                        current_item["url"] = json!(&after_href[..href_end]);
                    }
                }
            } else if let Some(pub_date) = extract_xml_content(line, "pubDate") {
                current_item["time"] = json!(pub_date);
            } else if let Some(published) = extract_xml_content(line, "published") {
                current_item["time"] = json!(published);
            }
        }
        
        if news_items.len() >= 5 {
            break;
        }
    }
    
    Ok(news_items)
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

fn clean_html(text: &str) -> String {
    // Remove CDATA
    let text = text.replace("<![CDATA[", "").replace("]]>", "");
    
    // Basic HTML entity decoding
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .trim()
        .to_string()
}

fn get_fallback_news() -> Vec<Value> {
    vec![
        json!({
            "title": "Kubernetes 1.29 Released with Enhanced Security Features",
            "url": "https://kubernetes.io/blog/2024/12/11/kubernetes-v1-29-release/",
            "source": "kubernetes",
            "icon": "📰",
            "time": "2024-12-11"
        }),
        json!({
            "title": "Docker Desktop 4.26 Introduces New Container Management Tools",
            "url": "https://www.docker.com/blog/docker-desktop-4-26/",
            "source": "docker",
            "icon": "🐳",
            "time": "2024-12-10"
        })
    ]
}
