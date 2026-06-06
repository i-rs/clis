pub(super) fn check_reminders() -> Option<String> {
    let output = i_rs_claw_core::utils::run_cli_command("i-rs", &["remind", "list", "--json"], 10).ok()?;

    let parsed: serde_json::Value = serde_json::from_str(&output).ok()?;

    let items = parsed.get("data")?.as_array()?;

    let due: Vec<String> = items
        .iter()
        .filter(|item| {
            let is_done = item
                .get("is_done")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if is_done {
                return false;
            }
            let days = item
                .get("days_until_event")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);
            days <= 0
        })
        .map(|item| {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("未知");
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let days = item
                .get("days_until_event")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            match title {
                Some(t) => {
                    if days == 0 {
                        format!("  - {}「{}」（今天到期）", t, name)
                    } else {
                        format!("  - {}「{}」（已过期 {} 天）", t, name, days.abs())
                    }
                }
                None => {
                    if days == 0 {
                        format!("  - {}（今天到期）", name)
                    } else {
                        format!("  - {}（已过期 {} 天）", name, days.abs())
                    }
                }
            }
        })
        .collect();

    if due.is_empty() {
        return None;
    }

    Some(due.join("\n"))
}

pub(super) fn notify_reminders(count: usize) {
    if count == 0 {
        return;
    }
    if cfg!(target_os = "macos") {
        let msg = format!("你有 {} 个待处理提醒", count);
        notify_macos("i-rs-claw 提醒", &msg);
    }
}

fn notify_macos(title: &str, message: &str) {
    let safe_title = title.replace('\\', "\\\\").replace('"', "\\\"");
    let safe_message = message.replace('\\', "\\\\").replace('"', "\\\"");
    let _ = std::process::Command::new("osascript")
        .args([
            "-e",
            &format!(
                r###"display notification "{}" with title "{}""###,
                safe_message, safe_title
            ),
        ])
        .output();
}
