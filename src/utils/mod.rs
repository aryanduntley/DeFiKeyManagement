use anyhow::Result;
use std::io::{self, Write};

pub fn confirm_action(message: &str) -> Result<bool> {
    print!("{} [y/N]: ", message);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let confirmed = input.trim().to_lowercase();
    Ok(confirmed == "y" || confirmed == "yes")
}

pub fn truncate_address(address: &str, start_chars: usize, end_chars: usize) -> String {
    if address.len() <= start_chars + end_chars + 3 {
        address.to_string()
    } else {
        format!("{}...{}", 
                &address[..start_chars], 
                &address[address.len()-end_chars..])
    }
}

pub fn format_table_row(columns: &[&str], widths: &[usize]) -> String {
    let mut row = String::new();
    for (i, (col, width)) in columns.iter().zip(widths).enumerate() {
        if i > 0 {
            row.push(' ');
        }
        row.push_str(&format!("{:<width$}", col, width = width));
    }
    row
}

pub fn print_table_separator(total_width: usize) {
    println!("{}", "-".repeat(total_width));
}

pub fn sanitize_label(label: &str) -> String {
    label.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_address() {
        let long_address = "0x1234567890abcdef1234567890abcdef12345678";
        let truncated = truncate_address(long_address, 6, 6);
        assert_eq!(truncated, "0x1234...345678");
        
        let short_address = "0x123";
        let not_truncated = truncate_address(short_address, 6, 6);
        assert_eq!(not_truncated, "0x123");
    }

    #[test]
    fn test_sanitize_label() {
        assert_eq!(sanitize_label("My Wallet!@#"), "My Wallet");
        assert_eq!(sanitize_label("  test_label-123  "), "test_label-123");
    }
}