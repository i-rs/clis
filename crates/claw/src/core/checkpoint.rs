use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub session_id: String,
    pub round: u32,
    pub messages: Vec<Value>,
    pub tool_results: HashMap<String, String>,
    pub timestamp: i64,
}

impl Checkpoint {
    pub fn new(session_id: &str, round: u32, messages: Vec<Value>) -> Self {
        Self {
            id: format!("cp_{}_{}", session_id, round),
            session_id: session_id.to_string(),
            round,
            messages,
            tool_results: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    #[allow(dead_code)]
    pub fn with_tool_results(mut self, results: HashMap<String, String>) -> Self {
        self.tool_results = results;
        self
    }

    #[allow(dead_code)]
    pub fn restore_messages(&self) -> Vec<Value> {
        self.messages.clone()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CheckpointStore {
    checkpoints: Vec<Checkpoint>,
    max_checkpoints: usize,
}

impl CheckpointStore {
    pub fn new(max_checkpoints: usize) -> Self {
        Self {
            checkpoints: Vec::new(),
            max_checkpoints,
        }
    }

    pub fn save(&mut self, checkpoint: Checkpoint) {
        if self.checkpoints.len() >= self.max_checkpoints {
            self.checkpoints.remove(0);
        }
        tracing::debug!(
            checkpoint_id = %checkpoint.id,
            round = %checkpoint.round,
            msg_count = %checkpoint.messages.len(),
            "Checkpoint saved"
        );
        self.checkpoints.push(checkpoint);
    }

    #[allow(dead_code)]
    pub fn latest(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }

    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|c| c.id == id)
    }

    #[allow(dead_code)]
    pub fn for_round(&self, round: u32) -> Option<&Checkpoint> {
        self.checkpoints
            .iter()
            .find(|c| c.round == round)
    }

    #[allow(dead_code)]
    pub fn list(&self) -> Vec<(String, u32, i64)> {
        self.checkpoints
            .iter()
            .map(|c| (c.id.clone(), c.round, c.timestamp))
            .collect()
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.checkpoints.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_checkpoint_create() {
        let cp = Checkpoint::new("sess1", 1, vec![json!({"role": "user", "content": "hi"})]);
        assert_eq!(cp.round, 1);
        assert_eq!(cp.messages.len(), 1);
    }

    #[test]
    fn test_checkpoint_restore() {
        let cp = Checkpoint::new("sess1", 1, vec![json!({"role": "user", "content": "hi"})]);
        let msgs = cp.restore_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0]["content"], "hi");
    }

    #[test]
    fn test_checkpoint_with_tool_results() {
        let mut results = HashMap::new();
        results.insert("weight".to_string(), "70kg".to_string());
        let cp = Checkpoint::new("sess1", 1, vec![])
            .with_tool_results(results);
        assert_eq!(cp.tool_results.get("weight"), Some(&"70kg".to_string()));
    }

    #[test]
    fn test_store_save_and_get() {
        let mut store = CheckpointStore::new(5);
        let cp = Checkpoint::new("sess1", 1, vec![json!({"role": "user", "content": "hi"})]);
        let id = cp.id.clone();
        store.save(cp);
        assert!(store.get(&id).is_some());
        assert!(store.latest().is_some());
    }

    #[test]
    fn test_store_eviction() {
        let mut store = CheckpointStore::new(2);
        store.save(Checkpoint::new("s", 1, vec![]));
        store.save(Checkpoint::new("s", 2, vec![]));
        store.save(Checkpoint::new("s", 3, vec![]));
        assert_eq!(store.list().len(), 2);
        assert_eq!(store.latest().unwrap().round, 3);
    }

    #[test]
    fn test_store_for_round() {
        let mut store = CheckpointStore::new(5);
        store.save(Checkpoint::new("s", 1, vec![]));
        store.save(Checkpoint::new("s", 2, vec![]));
        assert!(store.for_round(1).is_some());
        assert!(store.for_round(3).is_none());
    }

    #[test]
    fn test_store_clear() {
        let mut store = CheckpointStore::new(5);
        store.save(Checkpoint::new("s", 1, vec![]));
        store.clear();
        assert!(store.list().is_empty());
    }
}
