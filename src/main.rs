use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::env;
use std::time::Duration;

const NOTION_API_BASE: &str = "https://api.notion.com/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

fn get_api_version() -> String {
    env::var("NOTION_API_VERSION").unwrap_or_else(|_| "2022-06-28".to_string())
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
}

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

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Notion-Version", &self.api_version)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .context("Failed to send search request")?
                .error_for_status()
                .context("Notion API returned an error")?;

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

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", &self.api_version)
            .send()
            .context("Failed to get page")?
            .error_for_status()
            .context("Failed to fetch page from Notion")?;

        let result: serde_json::Value = response.json().context("Failed to parse response")?;
        Ok(result)
    }

    fn get_blocks(&self, page_id: &str) -> Result<Vec<serde_json::Value>> {
        let page_id = normalize_page_id(page_id)?;
        let url = format!("{}/blocks/{}/children", NOTION_API_BASE, page_id);
        let mut all_blocks = Vec::new();
        let mut start_cursor: Option<String> = None;

        loop {
            let mut request_url = url.clone();
            if let Some(cursor) = &start_cursor {
                request_url = format!("{}?start_cursor={}", url, cursor);
            }

            let response = self
                .client
                .get(&request_url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Notion-Version", &self.api_version)
                .send()
                .context("Failed to get blocks")?
                .error_for_status()
                .context("Failed to fetch blocks from Notion")?;

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

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", &self.api_version)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .context("Failed to create page")?
            .error_for_status()
            .context("Failed to create page in Notion")?;

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

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", &self.api_version)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .context("Failed to append blocks")?
            .error_for_status()
            .context("Failed to append blocks to Notion")?;

        let result: serde_json::Value = response.json().context("Failed to parse response")?;
        Ok(result)
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
