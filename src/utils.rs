use anyhow::{bail, Context, Result};
use std::env;

pub const NOTION_API_BASE: &str = "https://api.notion.com/v1";
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const MAX_RETRIES: u32 = 3;
pub const DEFAULT_RETRY_DELAY_SECS: u64 = 1;

pub fn get_api_version() -> String {
    env::var("NOTION_API_VERSION").unwrap_or_else(|_| "2025-09-03".to_string())
}

pub fn get_api_key() -> Result<String> {
    let _ = dotenvy::dotenv();
    env::var("NOTION_API_KEY")
        .context("NOTION_API_KEY not found. Set it in .env or as environment variable.")
}

/// Normalize page ID: remove dashes, validate format
pub fn normalize_page_id(id: &str) -> Result<String> {
    let clean: String = id.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    
    if clean.len() != 32 {
        bail!(
            "Invalid page ID '{}': expected 32 hex characters, got {}",
            id,
            clean.len()
        );
    }
    
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &clean[0..8],
        &clean[8..12],
        &clean[12..16],
        &clean[16..20],
        &clean[20..32]
    ))
}
