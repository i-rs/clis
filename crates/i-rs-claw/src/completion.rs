/// Tab completion for tool names and command names.
use crate::tools::index::TOOL_INDEX;

/// Common commands shared across most tools.
const COMMON_COMMANDS: &[&str] = &["add", "list", "get", "update", "delete"];

/// Tool-specific extra commands beyond the common ones.
const EXTRA_COMMANDS: &[(&str, &[&str])] = &[
    ("weight", &["stats", "chart"]),
    ("height", &["stats"]),
    ("sleep", &["stats"]),
    ("mood", &["calendar"]),
    ("exercise", &["stats"]),
    ("cycling", &["stats"]),
    ("budget", &["expense", "stats"]),
    ("invest", &["stats"]),
    ("debt", &["pay", "stats"]),
    ("invoice", &["stats"]),
    ("tax", &["stats"]),
    ("goal", &["deposit", "milestone", "stats"]),
    ("gift", &["stats"]),
    ("movie", &["stats"]),
    ("podcast", &["stats"]),
    ("read", &["stats"]),
    ("article", &["stats"]),
    ("vocab", &["quiz", "stats"]),
    ("birthday", &["stats"]),
    ("event", &["stats"]),
    ("contact", &["remind", "stats"]),
    ("car", &["fuel", "maintain", "stats"]),
    ("project", &["milestone", "stats"]),
    ("time", &["start", "stop", "report", "stats"]),
    ("deploy", &["rollback", "stats"]),
    ("vision", &["stats"]),
    ("appliance", &["stats"]),
    ("run", &["plan", "stats"]),
    ("habit", &["checkin", "streak"]),
    ("todo", &["done"]),
    ("remind", &["done"]),
    ("server", &["suggest"]),
    ("grocery", &["purchase", "clear"]),
    ("pig", &[]),
    ("dose", &[]),
    ("fast", &[]),
    ("cycle", &[]),
    ("sit", &[]),
    ("allergy", &[]),
    ("cal", &[]),
    ("meal", &[]),
    ("water", &[]),
    ("step", &[]),
    ("kv", &[]),
    ("keys", &[]),
    ("password", &[]),
    ("domain", &[]),
    ("bookmark", &[]),
    ("note", &[]),
    ("quote", &[]),
    ("snippet", &[]),
    ("spark", &[]),
    ("bestby", &[]),
    ("sub", &[]),
    ("recur", &[]),
    ("ledger", &[]),
    ("sheet", &[]),
    ("toothbrush", &[]),
    ("towel", &[]),
    ("bed", &[]),
    ("ac", &[]),
    ("filter", &[]),
    ("purify", &[]),
    ("feedpet", &[]),
    ("petbath", &[]),
    ("walkdog", &[]),
    ("aqua", &[]),
    ("want", &[]),
    ("contact", &["remind", "stats"]),
    ("birthday", &["stats"]),
    ("event", &["stats"]),
];

/// Get completions for the current input context.
/// Returns matching completions based on cursor position.
pub fn get_completions(input: &str, cursor: usize) -> Vec<String> {
    let input_before = &input[..cursor.min(input.len())];

    // Parse: "i-rs weight l" → tool = "weight", prefix = "l"
    // Parse: "weight l" → tool = "weight", prefix = "l"
    // Parse: "l" → prefix = "l" (could be tool)
    // Parse: "weight" → exact tool match, suggest commands

    let trimmed = input_before.trim();
    if trimmed.is_empty() {
        // Show all tool names
        return TOOL_INDEX.iter().map(|(name, _)| name.to_string()).collect();
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    match parts.len() {
        0 => {
            // Show all tool names
            TOOL_INDEX.iter().map(|(name, _)| name.to_string()).collect()
        }
        1 => {
            let word = parts[0];
            // Check if it's a complete tool name
            let is_complete_tool = TOOL_INDEX.iter().any(|(name, _)| *name == word);
            if is_complete_tool {
                // Suggest common commands for this tool
                get_tool_commands(word)
            } else {
                // Suggest tool names matching the prefix
                get_tool_completions(word)
            }
        }
        _ => {
            // Two or more words: "tool command_partial"
            let tool = parts[0];
            let prefix = parts[1..].join(" ");
            if TOOL_INDEX.iter().any(|(name, _)| *name == tool) {
                get_command_completions(tool, &prefix)
            } else {
                Vec::new()
            }
        }
    }
}

/// Get tool names matching a prefix.
fn get_tool_completions(prefix: &str) -> Vec<String> {
    let lower = prefix.to_lowercase();
    let mut matches: Vec<String> = TOOL_INDEX
        .iter()
        .filter(|(name, _)| name.starts_with(&lower))
        .map(|(name, _)| name.to_string())
        .collect();
    matches.sort();
    matches
}

/// Get commands for a tool matching a prefix.
fn get_command_completions(tool: &str, prefix: &str) -> Vec<String> {
    let commands = get_tool_commands(tool);
    if prefix.is_empty() {
        return commands;
    }
    let lower = prefix.to_lowercase();
    commands
        .into_iter()
        .filter(|cmd| cmd.starts_with(&lower))
        .collect()
}

/// Get all known commands (common + extra) for a tool.
fn get_tool_commands(tool: &str) -> Vec<String> {
    let mut cmds: Vec<String> = COMMON_COMMANDS.iter().map(|s| s.to_string()).collect();

    for &(name, extra) in EXTRA_COMMANDS {
        if name == tool {
            for cmd in extra {
                if !cmds.contains(&cmd.to_string()) {
                    cmds.push(cmd.to_string());
                }
            }
            break;
        }
    }
    cmds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_completions_prefix() {
        let results = get_completions("we", 2);
        assert!(results.contains(&"weight".to_string()));
        assert!(!results.contains(&"mood".to_string()));
    }

    #[test]
    fn test_tool_completions_exact_tool_shows_commands() {
        let results = get_completions("weight", 6);
        assert!(results.contains(&"add".to_string()));
        assert!(results.contains(&"list".to_string()));
        assert!(results.contains(&"stats".to_string()));
        assert!(results.contains(&"chart".to_string()));
    }

    #[test]
    fn test_command_completions_with_prefix() {
        let results = get_completions("weight st", 9);
        assert!(results.contains(&"stats".to_string()));
        assert!(!results.contains(&"add".to_string()));
    }

    #[test]
    fn test_empty_input_shows_all_tools() {
        let results = get_completions("", 0);
        assert!(results.len() >= 60);
        assert!(results.contains(&"kv".to_string()));
        assert!(results.contains(&"mood".to_string()));
    }

    #[test]
    fn test_unknown_tool() {
        let results = get_completions("nonexistent too", 16);
        assert!(results.is_empty());
    }
}
