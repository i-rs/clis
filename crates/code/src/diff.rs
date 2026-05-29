use similar::{ChangeTag, TextDiff};

#[allow(dead_code)]
pub struct DiffOutput {
    pub old: String,
    pub new: String,
    pub patch: String,
    pub lines_added: usize,
    pub lines_removed: usize,
}

pub fn diff_text(old: &str, new: &str) -> DiffOutput {
    let diff = TextDiff::from_lines(old, new);

    let mut patch = String::new();
    let mut lines_added = 0;
    let mut lines_removed = 0;

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => {
                lines_removed += 1;
                "-"
            }
            ChangeTag::Insert => {
                lines_added += 1;
                "+"
            }
            ChangeTag::Equal => " ",
        };
        patch.push_str(&format!("{}{}", sign, change.value()));
    }

    DiffOutput {
        old: old.to_string(),
        new: new.to_string(),
        patch,
        lines_added,
        lines_removed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_text_no_change() {
        let text = "hello\nworld\n";
        let result = diff_text(text, text);
        assert_eq!(result.lines_added, 0);
        assert_eq!(result.lines_removed, 0);
    }

    #[test]
    fn test_diff_text_add_line() {
        let old = "hello\n";
        let new = "hello\nworld\n";
        let result = diff_text(old, new);
        assert_eq!(result.lines_added, 1);
    }

    #[test]
    fn test_diff_text_remove_line() {
        let old = "hello\nworld\n";
        let new = "hello\n";
        let result = diff_text(old, new);
        assert_eq!(result.lines_removed, 1);
    }

    #[test]
    fn test_diff_text_modify() {
        let old = "hello\nworld\n";
        let new = "hello\nrust\n";
        let result = diff_text(old, new);
        assert!(result.lines_added > 0 || result.lines_removed > 0);
        assert!(result.patch.contains("rust"));
    }
}
