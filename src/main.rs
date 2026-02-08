mod cli;
mod client;
mod commands;
mod render;
mod utils;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::io::{self, Write};

use cli::{Cli, Commands};
use client::NotionClient;
use commands::*;
use utils::{get_api_key, get_config_path, load_config, save_config, Config};

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle commands that don't need API key first
    match &cli.command {
        Commands::Init { api_key } => {
            return handle_init(api_key.clone());
        }
        Commands::Config => {
            return handle_config_with_cli_key(cli.api_key.as_deref());
        }
        _ => {}
    }
    
    // Get API key with priority: CLI arg > env var > config file
    let api_key = match get_api_key(cli.api_key.as_deref()) {
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
        Commands::Init { .. } | Commands::Config => unreachable!(),
        Commands::Search { query, limit } => handle_search(&client, &query, limit),
        Commands::Read { page_id } => handle_read(&client, &page_id),
        Commands::Create { parent, title, content } => {
            handle_create(&client, &parent, &title, content.as_deref())
        }
        Commands::Append { page_id, content } => handle_append(&client, &page_id, &content),
        Commands::AppendCode { page_id, code, language } => {
            handle_append_code(&client, &page_id, &code, &language)
        }
        Commands::AppendBookmark { page_id, url, caption } => {
            handle_append_bookmark(&client, &page_id, &url, caption.as_deref())
        }
        Commands::Update { page_id, title, icon } => {
            handle_update(&client, &page_id, title.as_deref(), icon.as_deref())
        }
        Commands::Delete { page_id } => handle_delete(&client, &page_id),
        Commands::Query { database_id, filter, sort, direction, limit } => {
            handle_query(&client, &database_id, filter.as_deref(), sort.as_deref(), &direction, limit)
        }
        Commands::DeleteBlock { block_id } => handle_delete_block(&client, &block_id),
        Commands::AppendHeading { page_id, text, level } => {
            handle_append_heading(&client, &page_id, &text, level)
        }
        Commands::AppendDivider { page_id } => handle_append_divider(&client, &page_id),
        Commands::AppendList { page_id, items } => handle_append_list(&client, &page_id, &items),
        Commands::AppendLink { page_id, prefix, link_text, url, suffix } => {
            handle_append_link(&client, &page_id, prefix.as_deref(), &link_text, &url, suffix.as_deref())
        }
        Commands::GetBlockIds { page_id } => handle_get_block_ids(&client, &page_id),
        Commands::Move { page_id, parent, delete } => {
            handle_move(&client, &page_id, &parent, delete)
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "✗".red(), e);
        std::process::exit(1);
    }

    Ok(())
}

fn handle_init(api_key: Option<String>) -> Result<()> {
    let key = if let Some(k) = api_key {
        k
    } else {
        // Prompt for API key
        print!("{} Enter your Notion API key: ", "→".blue());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };
    
    if key.is_empty() {
        eprintln!("{} API key cannot be empty", "✗".red());
        std::process::exit(1);
    }
    
    // Validate key format (should start with secret_ or ntn_)
    if !key.starts_with("secret_") && !key.starts_with("ntn_") {
        eprintln!("{} Warning: API key should start with 'secret_' or 'ntn_'", "⚠".yellow());
    }
    
    // Save to config
    let config = Config {
        api_key: Some(key),
        timeout: None,
    };
    save_config(&config)?;
    
    let path = get_config_path().unwrap();
    println!("{} Config saved to {}", "✓".green(), path.display());
    println!("  You can now use notion-cli commands without setting NOTION_API_KEY");
    
    Ok(())
}

fn handle_config_with_cli_key(cli_api_key: Option<&str>) -> Result<()> {
    let config = load_config();
    let path = get_config_path();
    
    println!("{}", "Notion CLI Configuration".blue().bold());
    println!();
    
    if let Some(p) = &path {
        println!("Config file: {}", p.display());
    }
    println!();
    
    // API key status - show source based on priority order (matching get_api_key)
    print!("API key: ");
    
    // Check in priority order: CLI > env > config > .env
    if let Some(key) = cli_api_key {
        let masked = mask_api_key(key);
        println!("{} (from --api-key)", masked.green());
    } else if let Ok(key) = std::env::var("NOTION_API_KEY") {
        let masked = mask_api_key(&key);
        println!("{} (from environment)", masked.green());
    } else if let Some(key) = &config.api_key {
        let masked = mask_api_key(key);
        println!("{} (from config)", masked.green());
    } else {
        // Check .env as fallback
        if dotenvy::dotenv().is_ok() {
            if let Ok(key) = std::env::var("NOTION_API_KEY") {
                let masked = mask_api_key(&key);
                println!("{} (from .env)", masked.green());
            } else {
                println!("{}", "not set".red());
            }
        } else {
            println!("{}", "not set".red());
        }
    }
    
    println!();
    println!("{}", "Priority order:".dimmed());
    println!("  1. --api-key option");
    println!("  2. NOTION_API_KEY environment variable");
    println!("  3. ~/.config/notion-cli/config.toml");
    println!("  4. .env file (backward compatibility)");
    
    // Timeout
    if let Some(t) = config.timeout {
        println!("\nTimeout: {}s", t);
    }
    
    Ok(())
}

fn mask_api_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() > 12 {
        let prefix: String = chars[..8].iter().collect();
        let suffix: String = chars[chars.len()-4..].iter().collect();
        format!("{}...{}", prefix, suffix)
    } else {
        "***".to_string()
    }
}
