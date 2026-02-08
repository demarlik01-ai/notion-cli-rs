use colored::Colorize;

pub fn extract_title(item: &serde_json::Value) -> String {
    if let Some(props) = item.get("properties") {
        if let Some(title_prop) = props.get("title").or(props.get("Name")) {
            if let Some(title_arr) = title_prop.get("title").and_then(|t| t.as_array()) {
                if let Some(first) = title_arr.first() {
                    if let Some(text) = first.get("plain_text").and_then(|t| t.as_str()) {
                        return text.to_string();
                    }
                }
            }
        }
    }

    if let Some(title_arr) = item.get("title").and_then(|t| t.as_array()) {
        if let Some(first) = title_arr.first() {
            if let Some(text) = first.get("plain_text").and_then(|t| t.as_str()) {
                return text.to_string();
            }
        }
    }

    "(Untitled)".to_string()
}

pub fn extract_property_value(prop: &serde_json::Value) -> Option<String> {
    if let Some(rich_text) = prop.get("rich_text").and_then(|r| r.as_array()) {
        let text: String = rich_text
            .iter()
            .filter_map(|rt| rt.get("plain_text").and_then(|t| t.as_str()))
            .collect();
        if !text.is_empty() {
            return Some(text);
        }
    }

    if let Some(select) = prop.get("select") {
        if let Some(name) = select.get("name").and_then(|n| n.as_str()) {
            return Some(name.to_string());
        }
    }

    if let Some(multi_select) = prop.get("multi_select").and_then(|m| m.as_array()) {
        let values: Vec<&str> = multi_select
            .iter()
            .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
            .collect();
        if !values.is_empty() {
            return Some(values.join(", "));
        }
    }

    if let Some(number) = prop.get("number") {
        if let Some(n) = number.as_f64() {
            return Some(n.to_string());
        }
    }

    if let Some(checkbox) = prop.get("checkbox").and_then(|c| c.as_bool()) {
        return Some(if checkbox { "✓" } else { "✗" }.to_string());
    }

    if let Some(date) = prop.get("date") {
        if let Some(start) = date.get("start").and_then(|s| s.as_str()) {
            return Some(start.to_string());
        }
    }

    if let Some(url) = prop.get("url").and_then(|u| u.as_str()) {
        return Some(url.to_string());
    }

    None
}

pub fn print_block(block: &serde_json::Value) {
    let block_type = block
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    match block_type {
        "paragraph" => {
            if let Some(text) = extract_rich_text(block, "paragraph") {
                println!("{}", text);
            }
        }
        "heading_1" => {
            if let Some(text) = extract_rich_text(block, "heading_1") {
                println!("\n{}", format!("# {}", text).bold());
            }
        }
        "heading_2" => {
            if let Some(text) = extract_rich_text(block, "heading_2") {
                println!("\n{}", format!("## {}", text).bold());
            }
        }
        "heading_3" => {
            if let Some(text) = extract_rich_text(block, "heading_3") {
                println!("\n{}", format!("### {}", text).bold());
            }
        }
        "bulleted_list_item" => {
            if let Some(text) = extract_rich_text(block, "bulleted_list_item") {
                println!("  • {}", text);
            }
        }
        "numbered_list_item" => {
            if let Some(text) = extract_rich_text(block, "numbered_list_item") {
                println!("  1. {}", text);
            }
        }
        "code" => {
            if let Some(text) = extract_rich_text(block, "code") {
                println!("```\n{}\n```", text.dimmed());
            }
        }
        "divider" => {
            println!("{}", "---".dimmed());
        }
        _ => {}
    }
}

pub fn extract_rich_text(block: &serde_json::Value, block_type: &str) -> Option<String> {
    let rich_text = block.get(block_type)?.get("rich_text")?.as_array()?;
    let text: String = rich_text
        .iter()
        .filter_map(|rt| rt.get("plain_text").and_then(|t| t.as_str()))
        .collect();

    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
