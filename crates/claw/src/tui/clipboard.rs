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
    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "macos") {
        ("pbcopy", &[])
    } else if cfg!(target_os = "linux") {
        if which_exists("wl-copy") {
            ("wl-copy", &[])
        } else if which_exists("xclip") {
            ("xclip", &["-selection", "clipboard"])
        } else {
            return false;
        }
    } else if cfg!(target_os = "windows") {
        ("clip.exe", &[])
    } else {
        return false;
    };

    let mut child = match std::process::Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let Some(mut stdin) = child.stdin.take() else {
        let _ = child.kill();
        return false;
    };

    use std::io::Write;
    if stdin.write_all(text.as_bytes()).is_err() {
        let _ = child.kill();
        return false;
    }

    drop(stdin);

    child.wait().map(|s| s.success()).unwrap_or(false)
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
