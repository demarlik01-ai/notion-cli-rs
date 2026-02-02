use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::env;
use std::time::Duration;

const NOTION_API_BASE: &str = "https://api.notion.com/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

fn get_api_version() -> String {
    env::var("NOTION_API_VERSION").unwrap_or_else(|_| "2025-09-03".to_string())
}

#[derive(Parser)]
#[command(name = "notion")]
#[command(about = "A simple Notion CLI tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Request timeout in seconds
    #[arg(long, default_value_t = DEFAULT_TIMEOUT_SECS, global = true)]
    timeout: u64,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for pages and databases
    Search {
        /// Search query
        query: String,
        /// Maximum results to fetch (handles pagination)
        #[arg(short, long, default_value_t = 100)]
        limit: usize,
    },
    /// Read a page content
    Read {
        /// Page ID
        page_id: String,
    },
    /// Create a new page
    Create {
        /// Parent page ID
        #[arg(short, long)]
        parent: String,
        /// Page title
        #[arg(short, long)]
        title: String,
        /// Page content (optional)
        #[arg(short, long)]
        content: Option<String>,
    },
    /// Append content to a page
    Append {
        /// Page ID
        page_id: String,
        /// Content to append
        content: String,
    },
    /// Update a page (title, icon)
    Update {
        /// Page ID
        page_id: String,
        /// New title
        #[arg(short, long)]
        title: Option<String>,
        /// New icon (emoji)
        #[arg(short, long)]
        icon: Option<String>,
    },
    /// Delete (archive) a page
    Delete {
        /// Page ID
        page_id: String,
    },
    /// Query a database
    Query {
        /// Database ID
        database_id: String,
        /// Filter by property (format: "PropertyName=value" or "PropertyName:type=value")
        /// Supported types: title, rich_text (default), select, checkbox, number
        #[arg(short, long)]
        filter: Option<String>,
        /// Sort by property
        #[arg(short, long)]
        sort: Option<String>,
        /// Sort direction (asc or desc)
        #[arg(long, default_value = "desc")]
        direction: String,
        /// Maximum results
        #[arg(short, long, default_value_t = 100)]
        limit: usize,
    },
}

const MAX_RETRIES: u32 = 3;
const DEFAULT_RETRY_DELAY_SECS: u64 = 1;

struct NotionClient {
    api_key: String,
    api_version: String,
    client: reqwest::blocking::Client,
}

impl NotionClient {
    fn new(api_key: String, timeout_secs: u64) -> Result<Self> {
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

    fn search(&self, query: &str, limit: usize) -> Result<Vec<serde_json::Value>> {
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

            // Check for more pages
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

    fn get_page(&self, page_id: &str) -> Result<serde_json::Value> {
        let page_id = normalize_page_id(page_id)?;
        let url = format!("{}/pages/{}", NOTION_API_BASE, page_id);

        let response = self.execute_with_retry(|| self.client.get(&url))?;
        let result: serde_json::Value = response.json().context("Failed to parse response")?;
        Ok(result)
    }

    fn get_blocks(&self, page_id: &str) -> Result<Vec<serde_json::Value>> {
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

    fn create_page(&self, parent_id: &str, title: &str, content: Option<&str>) -> Result<serde_json::Value> {
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

    fn append_blocks(&self, page_id: &str, content: &str) -> Result<serde_json::Value> {
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

    fn update_page(&self, page_id: &str, title: Option<&str>, icon: Option<&str>) -> Result<serde_json::Value> {
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

    fn delete_page(&self, page_id: &str) -> Result<serde_json::Value> {
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

    fn query_database(&self, database_id: &str, filter: Option<&str>, sort: Option<&str>, direction: &str, limit: usize) -> Result<Vec<serde_json::Value>> {
        // Early return for zero limit
        if limit == 0 {
            return Ok(Vec::new());
        }

        let database_id = normalize_page_id(database_id)?;
        let url = format!("{}/databases/{}/query", NOTION_API_BASE, database_id);
        let mut all_results = Vec::new();
        let mut start_cursor: Option<String> = None;

        loop {
            // Clamp page_size between 1 and 100 (Notion API requirement)
            let remaining = limit.saturating_sub(all_results.len());
            let page_size = remaining.clamp(1, 100);

            let mut body = serde_json::json!({
                "page_size": page_size
            });

            if let Some(cursor) = &start_cursor {
                body["start_cursor"] = serde_json::json!(cursor);
            }

            // Parse filter: "PropertyName:type=value" or "PropertyName=value" (defaults to rich_text)
            // Supported types: title, rich_text, select, checkbox, number
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

            // Add sorting
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

/// Normalize page ID: remove dashes, validate format
fn normalize_page_id(id: &str) -> Result<String> {
    let clean: String = id.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    
    if clean.len() != 32 {
        bail!(
            "Invalid page ID '{}': expected 32 hex characters, got {}",
            id,
            clean.len()
        );
    }
    
    // Return with dashes in standard UUID format
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &clean[0..8],
        &clean[8..12],
        &clean[12..16],
        &clean[16..20],
        &clean[20..32]
    ))
}

fn get_api_key() -> Result<String> {
    // Try .env file first
    let _ = dotenvy::dotenv();
    
    env::var("NOTION_API_KEY")
        .context("NOTION_API_KEY not found. Set it in .env or as environment variable.")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let api_key = match get_api_key() {
        Ok(key) => key,
        Err(e) => {
            eprintln!("{} {}", "✗".red(), e);
            std::process::exit(1);
        }
    };
    
    let client = match NotionClient::new(api_key, cli.timeout) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} Failed to initialize client: {}", "✗".red(), e);
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        Commands::Search { query, limit } => handle_search(&client, &query, limit),
        Commands::Read { page_id } => handle_read(&client, &page_id),
        Commands::Create { parent, title, content } => {
            handle_create(&client, &parent, &title, content.as_deref())
        }
        Commands::Append { page_id, content } => handle_append(&client, &page_id, &content),
        Commands::Update { page_id, title, icon } => {
            handle_update(&client, &page_id, title.as_deref(), icon.as_deref())
        }
        Commands::Delete { page_id } => handle_delete(&client, &page_id),
        Commands::Query { database_id, filter, sort, direction, limit } => {
            handle_query(&client, &database_id, filter.as_deref(), sort.as_deref(), &direction, limit)
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "✗".red(), e);
        std::process::exit(1);
    }

    Ok(())
}

fn handle_search(client: &NotionClient, query: &str, limit: usize) -> Result<()> {
    println!("{} \"{}\"", "Searching:".blue(), query);
    
    let results = client.search(query, limit)?;
    println!("{} {} results found\n", "✓".green(), results.len());
    
    for item in &results {
        let object_type = item.get("object").and_then(|o| o.as_str()).unwrap_or("unknown");
        let id = item.get("id").and_then(|i| i.as_str()).unwrap_or("no-id");
        let title = extract_title(item);
        
        println!("  {} [{}] {}", "•".cyan(), object_type, title);
        println!("    ID: {}", id.dimmed());
    }
    
    Ok(())
}

fn handle_read(client: &NotionClient, page_id: &str) -> Result<()> {
    println!("{} {}", "Reading page:".blue(), page_id);
    
    let page = client.get_page(page_id)?;
    let blocks = client.get_blocks(page_id)?;
    
    let title = extract_title(&page);
    println!("\n{} {}\n", "Title:".green(), title);
    
    for block in &blocks {
        print_block(block);
    }
    
    Ok(())
}

fn handle_create(client: &NotionClient, parent: &str, title: &str, content: Option<&str>) -> Result<()> {
    println!("{} \"{}\"", "Creating page:".blue(), title);
    
    let result = client.create_page(parent, title, content)?;
    
    let id = result.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
    let url = result.get("url").and_then(|u| u.as_str());
    
    println!("{} Page created!", "✓".green());
    println!("  ID: {}", id);
    if let Some(u) = url {
        println!("  URL: {}", u);
    }
    
    Ok(())
}

fn handle_append(client: &NotionClient, page_id: &str, content: &str) -> Result<()> {
    println!("{} {}", "Appending to:".blue(), page_id);
    
    client.append_blocks(page_id, content)?;
    println!("{} Content appended!", "✓".green());
    
    Ok(())
}

fn handle_update(client: &NotionClient, page_id: &str, title: Option<&str>, icon: Option<&str>) -> Result<()> {
    if title.is_none() && icon.is_none() {
        bail!("At least one of --title or --icon must be specified");
    }

    println!("{} {}", "Updating page:".blue(), page_id);
    
    let result = client.update_page(page_id, title, icon)?;
    
    let new_title = extract_title(&result);
    println!("{} Page updated!", "✓".green());
    println!("  Title: {}", new_title);
    
    if let Some(icon_obj) = result.get("icon") {
        if let Some(emoji) = icon_obj.get("emoji").and_then(|e| e.as_str()) {
            println!("  Icon: {}", emoji);
        }
    }
    
    Ok(())
}

fn handle_delete(client: &NotionClient, page_id: &str) -> Result<()> {
    println!("{} {}", "Archiving page:".blue(), page_id);
    
    let result = client.delete_page(page_id)?;
    
    let archived = result.get("archived").and_then(|a| a.as_bool()).unwrap_or(false);
    if archived {
        println!("{} Page archived (moved to trash)!", "✓".green());
    } else {
        println!("{} Page status unclear", "⚠".yellow());
    }
    
    Ok(())
}

fn handle_query(client: &NotionClient, database_id: &str, filter: Option<&str>, sort: Option<&str>, direction: &str, limit: usize) -> Result<()> {
    println!("{} {}", "Querying database:".blue(), database_id);
    
    if let Some(f) = filter {
        println!("  Filter: {}", f);
    }
    if let Some(s) = sort {
        println!("  Sort: {} ({})", s, direction);
    }
    
    let results = client.query_database(database_id, filter, sort, direction, limit)?;
    println!("{} {} results found\n", "✓".green(), results.len());
    
    for item in &results {
        let id = item.get("id").and_then(|i| i.as_str()).unwrap_or("no-id");
        let title = extract_title(item);
        
        println!("  {} {}", "•".cyan(), title);
        println!("    ID: {}", id.dimmed());
        
        // Show some properties
        if let Some(props) = item.get("properties").and_then(|p| p.as_object()) {
            for (key, value) in props.iter().take(3) {
                if key == "title" || key == "Name" {
                    continue;
                }
                if let Some(prop_value) = extract_property_value(value) {
                    println!("    {}: {}", key.dimmed(), prop_value);
                }
            }
        }
    }
    
    Ok(())
}

fn extract_property_value(prop: &serde_json::Value) -> Option<String> {
    // Handle different property types
    if let Some(rich_text) = prop.get("rich_text").and_then(|r| r.as_array()) {
        let text: String = rich_text
            .iter()
            .filter_map(|rt| rt.get("plain_text").and_then(|t| t.as_str()))
            .collect();
        if !text.is_empty() {
            return Some(text);
        }
    }
    
    if let Some(select) = prop.get("select") {
        if let Some(name) = select.get("name").and_then(|n| n.as_str()) {
            return Some(name.to_string());
        }
    }
    
    if let Some(multi_select) = prop.get("multi_select").and_then(|m| m.as_array()) {
        let values: Vec<&str> = multi_select
            .iter()
            .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
            .collect();
        if !values.is_empty() {
            return Some(values.join(", "));
        }
    }
    
    if let Some(number) = prop.get("number") {
        if let Some(n) = number.as_f64() {
            return Some(n.to_string());
        }
    }
    
    if let Some(checkbox) = prop.get("checkbox").and_then(|c| c.as_bool()) {
        return Some(if checkbox { "✓" } else { "✗" }.to_string());
    }
    
    if let Some(date) = prop.get("date") {
        if let Some(start) = date.get("start").and_then(|s| s.as_str()) {
            return Some(start.to_string());
        }
    }
    
    if let Some(url) = prop.get("url").and_then(|u| u.as_str()) {
        return Some(url.to_string());
    }
    
    None
}

fn extract_title(item: &serde_json::Value) -> String {
    // Try different title locations
    if let Some(props) = item.get("properties") {
        // Database item or page with properties
        if let Some(title_prop) = props.get("title").or(props.get("Name")) {
            if let Some(title_arr) = title_prop.get("title").and_then(|t| t.as_array()) {
                if let Some(first) = title_arr.first() {
                    if let Some(text) = first.get("plain_text").and_then(|t| t.as_str()) {
                        return text.to_string();
                    }
                }
            }
        }
    }
    
    // Try title array directly (for databases)
    if let Some(title_arr) = item.get("title").and_then(|t| t.as_array()) {
        if let Some(first) = title_arr.first() {
            if let Some(text) = first.get("plain_text").and_then(|t| t.as_str()) {
                return text.to_string();
            }
        }
    }
    
    "(Untitled)".to_string()
}

fn print_block(block: &serde_json::Value) {
    let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
    
    match block_type {
        "paragraph" => {
            if let Some(text) = extract_rich_text(block, "paragraph") {
                println!("{}", text);
            }
        }
        "heading_1" => {
            if let Some(text) = extract_rich_text(block, "heading_1") {
                println!("\n{}", format!("# {}", text).bold());
            }
        }
        "heading_2" => {
            if let Some(text) = extract_rich_text(block, "heading_2") {
                println!("\n{}", format!("## {}", text).bold());
            }
        }
        "heading_3" => {
            if let Some(text) = extract_rich_text(block, "heading_3") {
                println!("\n{}", format!("### {}", text).bold());
            }
        }
        "bulleted_list_item" => {
            if let Some(text) = extract_rich_text(block, "bulleted_list_item") {
                println!("  • {}", text);
            }
        }
        "numbered_list_item" => {
            if let Some(text) = extract_rich_text(block, "numbered_list_item") {
                println!("  1. {}", text);
            }
        }
        "code" => {
            if let Some(text) = extract_rich_text(block, "code") {
                println!("```\n{}\n```", text.dimmed());
            }
        }
        "divider" => {
            println!("{}", "---".dimmed());
        }
        _ => {
            // Skip unknown block types silently
        }
    }
}

fn extract_rich_text(block: &serde_json::Value, block_type: &str) -> Option<String> {
    let rich_text = block.get(block_type)?.get("rich_text")?.as_array()?;
    let text: String = rich_text
        .iter()
        .filter_map(|rt| rt.get("plain_text").and_then(|t| t.as_str()))
        .collect();
    
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
