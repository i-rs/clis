pub(super) fn check_reminders() -> Option<String> {
    let output = std::process::Command::new("i-rs")
        .arg("remind")
        .arg("list")
        .arg("--json")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.as_ref()).ok()?;

    let items = parsed.get("data")?.as_array()?;

    let due: Vec<String> = items
        .iter()
        .filter(|item| {
            let is_done = item
                .get("is_done")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
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
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("未知");
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let _date = item
                .get("event_date")
                .and_then(|v| v.as_str())
                .unwrap_or("");
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

    notify_macos("i-rs-claw 提醒", &format!("你有 {} 个待处理提醒", due.len()));
    Some(due.join("\n"))
}

pub(super) fn notify_macos(title: &str, message: &str) {
    let _ = std::process::Command::new("osascript")
        .args(["-e", &format!(
            r###"display notification "{}" with title "{}""###,
            message, title
        )])
        .output();
}
