mod cli;
mod client;
mod commands;
mod render;
mod utils;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use cli::{Cli, Commands};
use client::NotionClient;
use commands::*;
use utils::get_api_key;

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
