use clap::{Parser, Subcommand};
use crate::utils::DEFAULT_TIMEOUT_SECS;

#[derive(Parser)]
#[command(name = "notion")]
#[command(about = "A simple Notion CLI tool", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Request timeout in seconds
    #[arg(long, default_value_t = DEFAULT_TIMEOUT_SECS, global = true)]
    pub timeout: u64,
}

#[derive(Subcommand)]
pub enum Commands {
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
