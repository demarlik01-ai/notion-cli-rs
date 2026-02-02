use anyhow::{bail, Result};
use colored::Colorize;

use crate::client::NotionClient;
use crate::render::{extract_title, extract_property_value, print_block};

pub fn handle_search(client: &NotionClient, query: &str, limit: usize) -> Result<()> {
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

pub fn handle_read(client: &NotionClient, page_id: &str) -> Result<()> {
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

pub fn handle_create(client: &NotionClient, parent: &str, title: &str, content: Option<&str>) -> Result<()> {
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

pub fn handle_append(client: &NotionClient, page_id: &str, content: &str) -> Result<()> {
    println!("{} {}", "Appending to:".blue(), page_id);
    
    client.append_blocks(page_id, content)?;
    println!("{} Content appended!", "✓".green());
    
    Ok(())
}

pub fn handle_update(client: &NotionClient, page_id: &str, title: Option<&str>, icon: Option<&str>) -> Result<()> {
    if title.is_none() && icon.is_none() {
        bail!("At least one of --title or --icon must be specified");
    }

    println!("{} {}", "Updating page:".blue(), page_id);
    
    let result = client.update_page(page_id, title, icon)?;
    
    let new_title = extract_title(&result);
    println!("{} Page updated!", "✓".green());
    println!("  Title: {}", new_title);
    
    if let Some(icon_obj) = result.get("icon") {
        if let Some(emoji) = icon_obj.get("emoji").and_then(|e| e.as_str()) {
            println!("  Icon: {}", emoji);
        }
    }
    
    Ok(())
}

pub fn handle_delete(client: &NotionClient, page_id: &str) -> Result<()> {
    println!("{} {}", "Archiving page:".blue(), page_id);
    
    let result = client.delete_page(page_id)?;
    
    let archived = result.get("archived").and_then(|a| a.as_bool()).unwrap_or(false);
    if archived {
        println!("{} Page archived (moved to trash)!", "✓".green());
    } else {
        println!("{} Page status unclear", "⚠".yellow());
    }
    
    Ok(())
}

pub fn handle_query(client: &NotionClient, database_id: &str, filter: Option<&str>, sort: Option<&str>, direction: &str, limit: usize) -> Result<()> {
    println!("{} {}", "Querying database:".blue(), database_id);
    
    if let Some(f) = filter {
        println!("  Filter: {}", f);
    }
    if let Some(s) = sort {
        println!("  Sort: {} ({})", s, direction);
    }
    
    let results = client.query_database(database_id, filter, sort, direction, limit)?;
    println!("{} {} results found\n", "✓".green(), results.len());
    
    for item in &results {
        let id = item.get("id").and_then(|i| i.as_str()).unwrap_or("no-id");
        let title = extract_title(item);
        
        println!("  {} {}", "•".cyan(), title);
        println!("    ID: {}", id.dimmed());
        
        if let Some(props) = item.get("properties").and_then(|p| p.as_object()) {
            for (key, value) in props.iter().take(3) {
                if key == "title" || key == "Name" {
                    continue;
                }
                if let Some(prop_value) = extract_property_value(value) {
                    println!("    {}: {}", key.dimmed(), prop_value);
                }
            }
        }
    }
    
    Ok(())
}
