pub(super) fn save_session_messages(
    session_mgr: &crate::session::SessionManager,
    session_id: &str,
    messages: &[crate::app::Message],
    api_messages: Option<&[serde_json::Value]>,
) {
    let records: Vec<serde_json::Value> =
        messages.iter().map(crate::app::message_to_jsonl).collect();
    session_mgr.save_all_messages(session_id, &records);
    if let Some(msgs) = api_messages {
        session_mgr.save_api_messages(session_id, msgs);
    }
    session_mgr.save_index();
}

pub(super) fn copy_to_clipboard(text: &str) -> bool {
    let cmd = if cfg!(target_os = "macos") {
        ("pbcopy", &[] as &[&str])
    } else if cfg!(target_os = "linux") {
        if std::process::Command::new("wl-copy").output().is_ok() {
            ("wl-copy", &[] as &[&str])
        } else {
            ("xclip", &["-selection", "clipboard"] as &[&str])
        }
    } else if cfg!(target_os = "windows") {
        ("clip", &[] as &[&str])
    } else {
        return false;
    };

    std::process::Command::new(cmd.0)
        .args(cmd.1)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .take()
                .and_then(|mut stdin| stdin.write_all(text.as_bytes()).ok());
            child.wait_with_output()
        })
        .map(|output| output.status.success())
        .unwrap_or(false)
}
