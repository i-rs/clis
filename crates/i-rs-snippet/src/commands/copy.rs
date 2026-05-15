use crate::presentation::print_success;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn handle_copy(name: String) -> Result<()> {
    let store = storage::load_store()?;

    let snippet = match storage::get_entry(&store, &name) {
        Some(s) => s,
        None => {
            anyhow::bail!("Snippet '{name}' not found");
        }
    };

    let code = snippet.code.join("\n");

    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("pbcopy");
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::null());
        
        let mut child = cmd.spawn()?;
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(code.as_bytes())?;
        }
        child.wait()?;
    }

    #[cfg(target_os = "linux")]
    {
        let mut cmd = Command::new("xclip");
        cmd.arg("-selection").arg("clipboard");
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::null());
        
        let mut child = cmd.spawn()?;
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(code.as_bytes())?;
        }
        child.wait()?;
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "echo", &code, "|", "clip"]);
        cmd.spawn()?.wait()?;
    }

    print_success(&format!("✓ Snippet '{}' copied to clipboard", name.green()));

    Ok(())
}
