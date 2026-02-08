use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

pub const NOTION_API_BASE: &str = "https://api.notion.com/v1";
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const MAX_RETRIES: u32 = 3;
pub const DEFAULT_RETRY_DELAY_SECS: u64 = 1;

/// Config file structure
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub api_key: Option<String>,
    pub timeout: Option<u64>,
}

/// Get config file path: ~/.config/notion-cli/config.toml
pub fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("notion-cli").join("config.toml"))
}

/// Load config from file
pub fn load_config() -> Config {
    get_config_path()
        .and_then(|path| fs::read_to_string(&path).ok())
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

/// Save config to file
pub fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path().context("Could not determine config directory")?;
    
    // Create directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }
    
    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, content).context("Failed to write config file")?;
    
    Ok(())
}

pub fn get_api_version() -> String {
    env::var("NOTION_API_VERSION").unwrap_or_else(|_| "2025-09-03".to_string())
}

/// Get API key with priority: CLI arg > env var > config file
/// Pass cli_api_key as None if not provided via CLI
pub fn get_api_key(cli_api_key: Option<&str>) -> Result<String> {
    // 1. CLI argument (highest priority)
    if let Some(key) = cli_api_key {
        return Ok(key.to_string());
    }
    
    // 2. Environment variable
    let _ = dotenvy::dotenv();
    if let Ok(key) = env::var("NOTION_API_KEY") {
        return Ok(key);
    }
    
    // 3. Config file
    let config = load_config();
    if let Some(key) = config.api_key {
        return Ok(key);
    }
    
    // None found - show helpful error
    let config_path = get_config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/.config/notion-cli/config.toml".to_string());
    
    bail!(
        "Notion API key not found.\n\n\
        Set it using one of these methods:\n\
        1. Run: notion-cli init\n\
        2. Set env: export NOTION_API_KEY=secret_xxx\n\
        3. Add to {}: api_key = \"secret_xxx\"\n\
        4. Use --api-key option",
        config_path
    )
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
