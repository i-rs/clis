pub fn print_content(content: &str, full: bool) {
    if full {
        println!("{}", content);
    } else if content.len() > 500 {
        println!("{}...", &content[..497]);
        println!("  (use --full to see all {} characters)", content.len());
    } else {
        println!("{}", content);
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
