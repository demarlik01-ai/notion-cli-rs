# notion-cli

[![Build Status](https://github.com/hyoseok/notion-cli-rs/workflows/CI/badge.svg)](https://github.com/hyoseok/notion-cli-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/notion-cli.svg)](https://crates.io/crates/notion-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast and simple Notion CLI written in Rust. Manage your Notion pages and databases from the terminal.

```bash
$ notion search "meeting notes"
✓ 3 results found

  • [page] Weekly Team Meeting
    ID: abc123...

  • [page] 1:1 Meeting Notes  
    ID: def456...
```

## Features

- 🔍 **Search** - Find pages and databases instantly
- 📖 **Read** - View page content with syntax highlighting
- ✏️ **Create** - Create new pages with content
- 📝 **Append** - Add text, code blocks, headings, lists, bookmarks
- 🔄 **Update** - Modify titles and icons
- 🗃️ **Query** - Filter and sort database entries
- 📦 **Move** - Relocate pages to different parents
- ⚡ **Fast** - Written in Rust, minimal overhead
- 🔄 **Auto-retry** - Handles rate limits automatically

## Installation

### From crates.io (Recommended)

```bash
cargo install notion-cli
```

### From source

```bash
git clone https://github.com/hyoseok/notion-cli-rs.git
cd notion-cli-rs
cargo install --path .
```

### Requirements

- Rust 1.70+ (for building from source)
- [Notion Integration Token](https://www.notion.so/my-integrations)

## Quick Start

### 1. Get your API key

1. Go to [Notion Integrations](https://www.notion.so/my-integrations)
2. Click "New integration"
3. Copy the "Internal Integration Token"
4. **Important**: Share your pages with the integration!

### 2. Configure

```bash
# Interactive setup (recommended)
notion init

# Or set environment variable
export NOTION_API_KEY=secret_xxxxx

# Or create config file manually
echo 'api_key = "secret_xxxxx"' > ~/.config/notion-cli/config.toml
```

### 3. Start using

```bash
notion search "my project"
notion read <page_id>
notion create --parent <page_id> --title "New Page"
```

## Configuration

API key is resolved in this order:
1. `--api-key` command line option
2. `NOTION_API_KEY` environment variable
3. `~/.config/notion-cli/config.toml`

```bash
# View current config
notion config

# Update config
notion init --api-key "secret_new_key"
```

## Usage

### Search

```bash
notion search "query"
notion search "project" --limit 10
```

### Read

```bash
notion read <page_id>
```

### Create

```bash
notion create --parent <parent_id> --title "Page Title"
notion create --parent <parent_id> --title "Page Title" --content "First paragraph"
```

### Append Content

```bash
# Text
notion append <page_id> "New paragraph"

# Code block
notion append-code <page_id> "console.log('hello')" --language javascript

# Heading
notion append-heading <page_id> "Section Title" --level 2

# Bulleted list
notion append-list <page_id> "Item 1" "Item 2" "Item 3"

# Bookmark
notion append-bookmark <page_id> --url "https://example.com"

# Divider
notion append-divider <page_id>
```

### Update

```bash
notion update <page_id> --title "New Title"
notion update <page_id> --icon "🚀"
notion update <page_id> --title "New Title" --icon "📝"
```

### Delete

```bash
notion delete <page_id>  # Moves to trash
```

### Query Database

```bash
# All entries
notion query <database_id>

# With filter
notion query <database_id> --filter "Status=Done"
notion query <database_id> --filter "Priority:select=High"

# With sort
notion query <database_id> --sort "Created" --direction desc

# Limit results
notion query <database_id> --limit 20
```

**Filter format:** `PropertyName=value` or `PropertyName:type=value`

**Supported types:** `title`, `rich_text`, `select`, `checkbox`, `number`

### Move Page

```bash
notion move <page_id> --parent <new_parent_id>
notion move <page_id> --parent <new_parent_id> --delete  # Archive original
```

### Other Commands

```bash
notion get-block-ids <page_id>    # List all block IDs
notion delete-block <block_id>    # Delete a specific block
```

### Global Options

```bash
notion --api-key <key> <command>  # Override API key
notion --timeout 60 <command>     # Custom timeout (default: 30s)
notion --help                     # Show help
notion --version                  # Show version
```

## API Version

Uses Notion API `2025-09-03` (latest).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.

---

Made with ❤️ and 🦀
