use similar::{ChangeTag, TextDiff};

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
