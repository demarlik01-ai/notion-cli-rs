use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::time::Duration;

use crate::utils::{
    get_api_version, normalize_page_id, NOTION_API_BASE, 
    MAX_RETRIES, DEFAULT_RETRY_DELAY_SECS
};

pub struct NotionClient {
    api_key: String,
    api_version: String,
    client: reqwest::blocking::Client,
}

impl NotionClient {
    pub fn new(api_key: String, timeout_secs: u64) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            api_key,
            api_version: get_api_version(),
            client,
        })
    }

    /// Execute a request with retry logic for rate limiting (429)
    fn execute_with_retry(&self, request_builder: impl Fn() -> reqwest::blocking::RequestBuilder) -> Result<reqwest::blocking::Response> {
        let mut retries = 0;
        
        loop {
            let response = request_builder()
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Notion-Version", &self.api_version)
                .send()
                .context("Failed to send request")?;

            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if retries >= MAX_RETRIES {
                    bail!("Rate limit exceeded after {} retries", MAX_RETRIES);
                }

                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_RETRY_DELAY_SECS);

                eprintln!(
                    "{} Rate limited. Waiting {} seconds before retry ({}/{})...",
                    "⚠".yellow(),
                    retry_after,
                    retries + 1,
                    MAX_RETRIES
                );

                std::thread::sleep(Duration::from_secs(retry_after));
                retries += 1;
                continue;
            }

            return response.error_for_status().context("Notion API returned an error");
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/search", NOTION_API_BASE);
        let mut all_results = Vec::new();
        let mut start_cursor: Option<String> = None;

        loop {
            let mut body = serde_json::json!({
                "query": query,
                "page_size": 100.min(limit - all_results.len())
            });

            if let Some(cursor) = &start_cursor {
                body["start_cursor"] = serde_json::json!(cursor);
            }

            let body_clone = body.clone();
            let url_clone = url.clone();
            let response = self.execute_with_retry(|| {
                self.client
                    .post(&url_clone)
                    .header("Content-Type", "application/json")
                    .json(&body_clone)
            })?;

            let result: serde_json::Value = response.json().context("Failed to parse response")?;
            
            if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
                all_results.extend(results.clone());
            }

            let has_more = result.get("has_more").and_then(|h| h.as_bool()).unwrap_or(false);
            if !has_more || all_results.len() >= limit {
                break;
            }

            start_cursor = result.get("next_cursor").and_then(|c| c.as_str()).map(String::from);
            if start_cursor.is_none() {
                break;
            }
        }

        Ok(all_results)
    }

    pub fn get_page(&self, page_id: &str) -> Result<serde_json::Value> {
        let page_id = normalize_page_id(page_id)?;
        let url = format!("{}/pages/{}", NOTION_API_BASE, page_id);

        let response = self.execute_with_retry(|| self.client.get(&url))?;
        let result: serde_json::Value = response.json().context("Failed to parse response")?;
        Ok(result)
    }

    pub fn get_blocks(&self, page_id: &str) -> Result<Vec<serde_json::Value>> {
        let page_id = normalize_page_id(page_id)?;
        let base_url = format!("{}/blocks/{}/children", NOTION_API_BASE, page_id);
        let mut all_blocks = Vec::new();
        let mut start_cursor: Option<String> = None;

        loop {
            let request_url = if let Some(cursor) = &start_cursor {
                format!("{}?start_cursor={}", base_url, cursor)
            } else {
                base_url.clone()
            };

            let response = self.execute_with_retry(|| self.client.get(&request_url))?;
            let result: serde_json::Value = response.json().context("Failed to parse response")?;
            
            if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
                all_blocks.extend(results.clone());
            }

            let has_more = result.get("has_more").and_then(|h| h.as_bool()).unwrap_or(false);
            if !has_more {
                break;
            }

            start_cursor = result.get("next_cursor").and_then(|c| c.as_str()).map(String::from);
            if start_cursor.is_none() {
                break;
            }
        }

        Ok(all_blocks)
    }

    pub fn create_page(&self, parent_id: &str, title: &str, content: Option<&str>) -> Result<serde_json::Value> {
        let parent_id = normalize_page_id(parent_id)?;
        let url = format!("{}/pages", NOTION_API_BASE);
        
        let mut children = vec![];
        if let Some(text) = content {
            children.push(serde_json::json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": { "content": text }
                    }]
                }
            }));
        }

        let body = serde_json::json!({
            "parent": { "page_id": parent_id },
            "properties": {
                "title": {
                    "title": [{
                        "text": { "content": title }
                    }]
                }
            },
            "children": children
        });

        let response = self.execute_with_retry(|| {
            self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
        })?;

        let result: serde_json::Value = response.json().context("Failed to parse response")?;
        Ok(result)
    }

    pub fn append_blocks(&self, page_id: &str, content: &str) -> Result<serde_json::Value> {
        let page_id = normalize_page_id(page_id)?;
        let url = format!("{}/blocks/{}/children", NOTION_API_BASE, page_id);

        let body = serde_json::json!({
            "children": [{
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": { "content": content }
                    }]
                }
            }]
        });

        let response = self.execute_with_retry(|| {
            self.client
                .patch(&url)
                .header("Content-Type", "application/json")
                .json(&body)
        })?;

        let result: serde_json::Value = response.json().context("Failed to parse response")?;
        Ok(result)
    }

    pub fn update_page(&self, page_id: &str, title: Option<&str>, icon: Option<&str>) -> Result<serde_json::Value> {
        let page_id = normalize_page_id(page_id)?;
        let url = format!("{}/pages/{}", NOTION_API_BASE, page_id);

        let mut body = serde_json::json!({});

        if let Some(new_title) = title {
            body["properties"] = serde_json::json!({
                "title": {
                    "title": [{
                        "text": { "content": new_title }
                    }]
                }
            });
        }

        if let Some(emoji) = icon {
            body["icon"] = serde_json::json!({
                "type": "emoji",
                "emoji": emoji
            });
        }

        let response = self.execute_with_retry(|| {
            self.client
                .patch(&url)
                .header("Content-Type", "application/json")
                .json(&body)
        })?;

        let result: serde_json::Value = response.json().context("Failed to parse response")?;
        Ok(result)
    }

    pub fn delete_page(&self, page_id: &str) -> Result<serde_json::Value> {
        let page_id = normalize_page_id(page_id)?;
        let url = format!("{}/pages/{}", NOTION_API_BASE, page_id);

        let body = serde_json::json!({
            "archived": true
        });

        let response = self.execute_with_retry(|| {
            self.client
                .patch(&url)
                .header("Content-Type", "application/json")
                .json(&body)
        })?;

        let result: serde_json::Value = response.json().context("Failed to parse response")?;
        Ok(result)
    }

    pub fn query_database(&self, database_id: &str, filter: Option<&str>, sort: Option<&str>, direction: &str, limit: usize) -> Result<Vec<serde_json::Value>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let database_id = normalize_page_id(database_id)?;
        let url = format!("{}/databases/{}/query", NOTION_API_BASE, database_id);
        let mut all_results = Vec::new();
        let mut start_cursor: Option<String> = None;

        loop {
            let remaining = limit.saturating_sub(all_results.len());
            let page_size = remaining.clamp(1, 100);

            let mut body = serde_json::json!({
                "page_size": page_size
            });

            if let Some(cursor) = &start_cursor {
                body["start_cursor"] = serde_json::json!(cursor);
            }

            if let Some(filter_str) = filter {
                if let Some((prop_part, value)) = filter_str.split_once('=') {
                    let (prop, filter_type) = if let Some((p, t)) = prop_part.split_once(':') {
                        (p.trim(), t.trim())
                    } else {
                        (prop_part.trim(), "rich_text")
                    };

                    let filter_value = match filter_type {
                        "title" => serde_json::json!({
                            "property": prop,
                            "title": { "contains": value.trim() }
                        }),
                        "select" => serde_json::json!({
                            "property": prop,
                            "select": { "equals": value.trim() }
                        }),
                        "checkbox" => serde_json::json!({
                            "property": prop,
                            "checkbox": { "equals": value.trim().to_lowercase() == "true" }
                        }),
                        "number" => {
                            let num: f64 = value.trim().parse().unwrap_or(0.0);
                            serde_json::json!({
                                "property": prop,
                                "number": { "equals": num }
                            })
                        },
                        _ => serde_json::json!({
                            "property": prop,
                            "rich_text": { "contains": value.trim() }
                        }),
                    };
                    body["filter"] = filter_value;
                }
            }

            if let Some(sort_prop) = sort {
                body["sorts"] = serde_json::json!([{
                    "property": sort_prop,
                    "direction": if direction == "asc" { "ascending" } else { "descending" }
                }]);
            }

            let body_clone = body.clone();
            let url_clone = url.clone();
            let response = self.execute_with_retry(|| {
                self.client
                    .post(&url_clone)
                    .header("Content-Type", "application/json")
                    .json(&body_clone)
            })?;

            let result: serde_json::Value = response.json().context("Failed to parse response")?;
            
            if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
                all_results.extend(results.clone());
            }

            let has_more = result.get("has_more").and_then(|h| h.as_bool()).unwrap_or(false);
            if !has_more || all_results.len() >= limit {
                break;
            }

            start_cursor = result.get("next_cursor").and_then(|c| c.as_str()).map(String::from);
            if start_cursor.is_none() {
                break;
            }
        }

        Ok(all_results)
    }
}
