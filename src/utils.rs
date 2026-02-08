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

/// Get API key with priority: CLI arg > env var > config file > .env (backward compat)
/// Pass cli_api_key as None if not provided via CLI
pub fn get_api_key(cli_api_key: Option<&str>) -> Result<String> {
    // 1. CLI argument (highest priority)
    if let Some(key) = cli_api_key {
        return Ok(key.to_string());
    }

    // 2. Environment variable (without loading .env)
    if let Ok(key) = env::var("NOTION_API_KEY") {
        return Ok(key);
    }

    // 3. Config file (~/.config/notion-cli/config.toml)
    let config = load_config();
    if let Some(key) = config.api_key {
        return Ok(key);
    }

    // 4. .env file (backward compatibility fallback)
    if dotenvy::dotenv().is_ok() {
        if let Ok(key) = env::var("NOTION_API_KEY") {
            return Ok(key);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_page_id_with_dashes() {
        let result = normalize_page_id("2fb74f32-4ab9-80f5-83df-c93c885072e7").unwrap();
        assert_eq!(result, "2fb74f32-4ab9-80f5-83df-c93c885072e7");
    }

    #[test]
    fn test_normalize_page_id_without_dashes() {
        let result = normalize_page_id("2fb74f324ab980f583dfc93c885072e7").unwrap();
        assert_eq!(result, "2fb74f32-4ab9-80f5-83df-c93c885072e7");
    }

    #[test]
    fn test_normalize_page_id_invalid() {
        let result = normalize_page_id("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            api_key: Some("ntn_test123".to_string()),
            timeout: Some(60),
        };

        let serialized = toml::to_string_pretty(&config).unwrap();
        assert!(serialized.contains("api_key = \"ntn_test123\""));
        assert!(serialized.contains("timeout = 60"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
api_key = "ntn_test456"
timeout = 45
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api_key, Some("ntn_test456".to_string()));
        assert_eq!(config.timeout, Some(45));
    }

    #[test]
    fn test_config_minimal() {
        // Only api_key, no timeout
        let toml_str = r#"api_key = "secret_xyz""#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api_key, Some("secret_xyz".to_string()));
        assert_eq!(config.timeout, None);
    }

    #[test]
    fn test_get_config_path() {
        let path = get_config_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("notion-cli"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }
}
