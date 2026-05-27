use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

pub struct Transport {
    tx: mpsc::UnboundedSender<String>,
    rx: mpsc::UnboundedReceiver<String>,
}

impl Transport {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<String> {
        self.tx.clone()
    }

    pub fn receiver(&mut self) -> &mut mpsc::UnboundedReceiver<String> {
        &mut self.rx
    }

    /// Send a JSON event to claw (write to stdout)
    pub fn send_event(event: &impl serde::Serialize) -> anyhow::Result<()> {
        let line = serde_json::to_string(event)?;
        println!("{}", line);
        Ok(())
    }

    /// Read a JSON line from stdin (blocking in spawned task)
    pub async fn read_line() -> anyhow::Result<String> {
        let stdin = tokio::io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        if let Some(line) = lines.next_line().await? {
            Ok(line)
        } else {
            anyhow::bail!("stdin closed")
        }
    }
}
