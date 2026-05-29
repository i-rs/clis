use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;

use tokio::sync::Mutex as AsyncMutex;

const MAX_OUTPUT: usize = 2_000_000;

struct Inner {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout_rx: mpsc::Receiver<String>,
    stderr_rx: mpsc::Receiver<String>,
}

impl Inner {
    fn spawn(cwd: &Path) -> anyhow::Result<Self> {
        let mut child = Command::new("sh")
            .arg("-s")
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("no stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| anyhow::anyhow!("no stderr"))?;

        let (stdout_tx, stdout_rx) = mpsc::channel::<String>();
        let (stderr_tx, stderr_rx) = mpsc::channel::<String>();

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        if stdout_tx.send(l).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        if stderr_tx.send(l).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            child: Some(child),
            stdin: Some(stdin),
            stdout_rx,
            stderr_rx,
        })
    }

    fn exec(&mut self, command: &str, timeout_secs: u64) -> anyhow::Result<String> {
        let stdin = self.stdin.as_mut().ok_or_else(|| anyhow::anyhow!("no stdin"))?;

        writeln!(stdin, "{}", command)?;
        writeln!(stdin, "echo __PTY_EXIT_$?")?;
        stdin.flush()?;

        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        let poll_interval = Duration::from_millis(50);
        let mut output = String::new();

        loop {
            if Instant::now() >= deadline {
                break;
            }

            match self.stdout_rx.recv_timeout(poll_interval) {
                Ok(line) => {
                    let trimmed = line.trim_end().to_string();
                    if trimmed.starts_with("__PTY_EXIT_") {
                        let extra = Duration::from_secs(5);
                        drain_remaining(&self.stdout_rx, &self.stderr_rx, &mut output, Instant::now() + extra);
                        return Ok(output);
                    }
                    if !output.is_empty() { output.push('\n'); }
                    output.push_str(&trimmed);
                    if output.len() > MAX_OUTPUT { output.truncate(MAX_OUTPUT); }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    while let Ok(line) = self.stderr_rx.try_recv() {
                        let trimmed = line.trim_end().to_string();
                        if !output.is_empty() { output.push('\n'); }
                        output.push_str(&trimmed);
                        if output.len() > MAX_OUTPUT { output.truncate(MAX_OUTPUT); }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        // Timeout: drain remaining lines so next call starts clean
        let drain_deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if Instant::now() > drain_deadline { break; }
            match self.stdout_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(line) => {
                    if line.trim_end() == "__PTY_EXIT_$?" { break; }
                    if line.trim_end().starts_with("__PTY_EXIT_") { break; }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        Ok(output)
    }
}

fn drain_remaining(
    stdout_rx: &mpsc::Receiver<String>,
    stderr_rx: &mpsc::Receiver<String>,
    output: &mut String,
    deadline: Instant,
) {
    let poll = Duration::from_millis(50);
    while Instant::now() < deadline {
        let mut got_any = false;
        while let Ok(line) = stdout_rx.try_recv() {
            got_any = true;
            let trimmed = line.trim_end().to_string();
            if !output.is_empty() { output.push('\n'); }
            output.push_str(&trimmed);
            if output.len() > MAX_OUTPUT { output.truncate(MAX_OUTPUT); }
        }
        while let Ok(line) = stderr_rx.try_recv() {
            got_any = true;
            let trimmed = line.trim_end().to_string();
            if !output.is_empty() { output.push('\n'); }
            output.push_str(&trimmed);
            if output.len() > MAX_OUTPUT { output.truncate(MAX_OUTPUT); }
        }
        if !got_any { break; }
        std::thread::sleep(poll);
    }
}

#[derive(Clone)]
pub struct PtySession {
    inner: Arc<Mutex<Inner>>,
}

impl PtySession {
    pub fn spawn(cwd: &Path) -> anyhow::Result<Self> {
        let inner = Inner::spawn(cwd)?;
        Ok(Self { inner: Arc::new(Mutex::new(inner)) })
    }

    pub async fn exec_async(&self, command: &str, timeout_secs: u64) -> anyhow::Result<String> {
        let command = command.to_string();
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = inner.lock().expect("pty mutex poisoned");
            guard.exec(&command, timeout_secs)
        })
        .await?
    }

    pub fn interrupt(&self) -> anyhow::Result<()> {
        let mut guard = self.inner.lock().expect("pty mutex poisoned");
        if let Some(child) = guard.child.as_mut() {
            child.kill()?;
        }
        guard.child = None;
        guard.stdin = None;
        Ok(())
    }
}

pub struct PtyManager {
    sessions: AsyncMutex<HashMap<String, PtySession>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self { sessions: AsyncMutex::new(HashMap::new()) }
    }

    pub async fn exec(&self, id: &str, command: &str, timeout_secs: u64, cwd: &Path) -> anyhow::Result<String> {
        let mut sessions = self.sessions.lock().await;
        if !sessions.contains_key(id) {
            let session = PtySession::spawn(cwd)?;
            sessions.insert(id.to_string(), session);
        }
        let session = sessions.get(id)
            .ok_or_else(|| anyhow::anyhow!("pty session '{}' not found", id))?
            .clone();
        session.exec_async(command, timeout_secs).await
    }

    pub async fn interrupt(&self, id: &str) -> anyhow::Result<()> {
        let sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get(id) {
            session.interrupt()
        } else {
            Err(anyhow::anyhow!("no such pty session: {}", id))
        }
    }
}
