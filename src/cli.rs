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
    /// Append a code block to a page
    AppendCode {
        /// Page ID
        page_id: String,
        /// Code content
        code: String,
        /// Programming language (e.g., rust, python, javascript)
        #[arg(short, long, default_value = "plain text")]
        language: String,
    },
    /// Append a bookmark to a page
    AppendBookmark {
        /// Page ID
        page_id: String,
        /// Bookmark URL
        url: String,
        /// Optional caption
        #[arg(short, long)]
        caption: Option<String>,
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
    /// Delete (archive) a block
    DeleteBlock {
        /// Block ID
        block_id: String,
    },
    /// Append a heading to a page
    AppendHeading {
        /// Page ID
        page_id: String,
        /// Heading text
        text: String,
        /// Heading level (1, 2, or 3)
        #[arg(short, long, default_value_t = 2)]
        level: u8,
    },
    /// Append a divider to a page
    AppendDivider {
        /// Page ID
        page_id: String,
    },
    /// Append a bulleted list to a page
    AppendList {
        /// Page ID
        page_id: String,
        /// List items (comma-separated)
        items: String,
    },
    /// Append a paragraph with a link
    AppendLink {
        /// Page ID
        page_id: String,
        /// Text before the link
        #[arg(long)]
        prefix: Option<String>,
        /// Link text
        #[arg(long)]
        link_text: String,
        /// Link URL
        #[arg(long)]
        url: String,
        /// Text after the link
        #[arg(long)]
        suffix: Option<String>,
    },
    /// Get block IDs for a page (for bulk operations)
    GetBlockIds {
        /// Page ID
        page_id: String,
    },
    /// Move a page to a new parent
    Move {
        /// Source page ID
        page_id: String,
        /// New parent page ID
        #[arg(short, long)]
        parent: String,
        /// Delete original page after copying
        #[arg(long, default_value_t = false)]
        delete: bool,
    },
}
