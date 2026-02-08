# Architecture

## Overview

notion-cli is a Rust CLI application that wraps the Notion REST API for terminal usage. It follows a modular architecture with 6 source files (~1,700 LOC total).

## Project Structure

```
notion-cli-rs/
├── src/
│   ├── main.rs        # Entry point, command routing, init/config handlers
│   ├── cli.rs         # CLI argument definitions (clap derive)
│   ├── client.rs      # NotionClient - HTTP client & API methods
│   ├── commands.rs    # Command handler functions
│   ├── render.rs      # Terminal output formatting
│   └── utils.rs       # Config management, helpers, constants
├── docs/
│   ├── ARCHITECTURE.md
│   ├── ARCHITECTURE-ko.md
│   └── API_COMPARISON.md
├── Cargo.toml
└── README.md
```

## Module Diagram

```
                    ┌──────────────┐
                    │   main.rs    │
                    │  - CLI parse │
                    │  - Routing   │
                    │  - init/cfg  │
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
       ┌───────────┐ ┌──────────┐ ┌──────────┐
       │  cli.rs   │ │commands.rs│ │ utils.rs │
       │  (clap)   │ │(handlers)│ │ (config) │
       └───────────┘ └────┬─────┘ └──────────┘
                          │
                          ▼
                   ┌─────────────┐
                   │ client.rs   │
                   │(NotionClient)│
                   └──────┬──────┘
                          │
                   ┌──────┴──────┐
                   ▼             ▼
            ┌───────────┐ ┌───────────┐
            │ render.rs │ │ Notion API│
            │ (output)  │ │  (REST)   │
            └───────────┘ └───────────┘
```

## Modules

### `cli.rs` — CLI Definitions

Defines the CLI structure using clap's derive API.

- `Cli` struct: global options (`--api-key`, `--timeout`)
- `Commands` enum: 18 subcommands (search, read, create, append, update, delete, query, move, init, config, etc.)

### `main.rs` — Entry Point & Routing

1. Parses CLI arguments
2. Handles `init` and `config` commands (no API key needed)
3. Resolves API key via priority chain
4. Initializes `NotionClient`
5. Routes to appropriate command handler

Also contains `handle_init()` and `handle_config_with_cli_key()`.

### `client.rs` — Notion API Client

`NotionClient` wraps reqwest's blocking HTTP client.

**Key features:**
- Bearer token authentication
- Notion-Version header (`2025-09-03`)
- Automatic pagination for search and block retrieval
- Auto-retry with exponential backoff on rate limits (HTTP 429)
- Rich text builder helpers (`plain`, `link`, `code_inline`, `bold`)

**API methods (16):**
| Method | HTTP | Endpoint |
|--------|------|----------|
| `search` | POST | `/search` |
| `get_page` | GET | `/pages/{id}` |
| `get_blocks` | GET | `/blocks/{id}/children` |
| `create_page` | POST | `/pages` |
| `append_blocks` | PATCH | `/blocks/{id}/children` |
| `update_page` | PATCH | `/pages/{id}` |
| `delete_page` | PATCH | `/pages/{id}` (archive) |
| `append_code_block` | PATCH | `/blocks/{id}/children` |
| `append_bookmark` | PATCH | `/blocks/{id}/children` |
| `delete_block` | DELETE | `/blocks/{id}` |
| `append_heading` | PATCH | `/blocks/{id}/children` |
| `append_rich_text` | PATCH | `/blocks/{id}/children` |
| `append_divider` | PATCH | `/blocks/{id}/children` |
| `append_bulleted_list` | PATCH | `/blocks/{id}/children` |
| `query_database` | POST | `/databases/{id}/query` |
| `move_page` | POST+PATCH | `/pages` + `/pages/{id}` |

### `commands.rs` — Command Handlers

Each handler function:
1. Validates and normalizes input (page IDs, etc.)
2. Calls `NotionClient` methods
3. Formats output via `render.rs`

16 handler functions corresponding to CLI subcommands.

### `render.rs` — Output Formatting

Terminal rendering with `colored` crate:
- `extract_title()` — extract title from Notion page/database objects
- `extract_rich_text()` — extract plain text from block rich_text arrays
- `extract_property_value()` — extract property values for database query results
- `print_block()` — format and print individual blocks by type

**Supported block types:** paragraph, heading (1-3), bulleted/numbered list, code, divider, bookmark, to-do

### `utils.rs` — Configuration & Helpers

**Config management:**
- `Config` struct: `api_key`, `timeout` (serialized as TOML)
- Config path: `~/.config/notion-cli/config.toml`
- `load_config()` / `save_config()` — TOML read/write

**API key resolution priority:**
1. `--api-key` CLI option
2. `NOTION_API_KEY` environment variable
3. `~/.config/notion-cli/config.toml`
4. `.env` file (backward compatibility)

**Other utilities:**
- `normalize_page_id()` — converts various ID formats to UUID
- `get_api_version()` — API version string

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing (derive) |
| `reqwest` | HTTP client (blocking, rustls-tls) |
| `serde` / `serde_json` | JSON serialization |
| `toml` | Config file parsing |
| `dirs` | XDG config directory resolution |
| `dotenvy` | .env file loading (legacy fallback) |
| `anyhow` | Error handling with context |
| `colored` | Terminal color output |

## Error Handling

- All functions return `anyhow::Result<T>`
- `.context()` on every API call for clear error messages
- Rate limit (HTTP 429): automatic retry with backoff (max 3 retries)
- `main()` catches all errors → prints with red `✗` → exits code 1

## Design Decisions

- **Blocking HTTP**: Async is unnecessary for a sequential CLI tool
- **Modular files**: Split from single file at ~800 LOC for maintainability
- **Global config**: XDG-compliant `~/.config/` over `.env` for portability
- **Auto-pagination**: Users never deal with cursors manually
- **No database item creation**: Pages only — keeps the API surface simple
