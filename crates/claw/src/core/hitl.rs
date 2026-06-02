use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ConfirmationLevel {
    Auto,
    Notify,
    Confirm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    pub tool_name: String,
    pub args: Value,
    pub risk_level: RiskLevel,
    pub description: String,
    pub auto_confirm_timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfirmationResponse {
    Approved,
    Denied { reason: String },
    Modified { args: Value },
}

pub struct HitlPolicy {
    auto_approve_tools: HashSet<String>,
    confirm_tools: HashSet<String>,
    deny_tools: HashSet<String>,
    risk_threshold: RiskLevel,
    dangerous_commands: HashSet<String>,
}

impl HitlPolicy {
    pub fn new() -> Self {
        let mut dangerous_commands = HashSet::new();
        for cmd in &["delete", "remove", "clear", "reset", "purge", "drop", "truncate"] {
            dangerous_commands.insert(cmd.to_string());
        }
        Self {
            auto_approve_tools: HashSet::new(),
            confirm_tools: HashSet::new(),
            deny_tools: HashSet::new(),
            risk_threshold: RiskLevel::Medium,
            dangerous_commands,
        }
    }

    pub fn auto_approve(mut self, tool: &str) -> Self {
        self.auto_approve_tools.insert(tool.to_string());
        self
    }

    pub fn require_confirm(mut self, tool: &str) -> Self {
        self.confirm_tools.insert(tool.to_string());
        self
    }

    pub fn deny(mut self, tool: &str) -> Self {
        self.deny_tools.insert(tool.to_string());
        self
    }

    pub fn with_risk_threshold(mut self, level: RiskLevel) -> Self {
        self.risk_threshold = level;
        self
    }

    pub fn check(&self, tool_name: &str, args: &Value) -> ConfirmationRequest {
        if self.deny_tools.contains(tool_name) {
            return ConfirmationRequest {
                tool_name: tool_name.to_string(),
                args: args.clone(),
                risk_level: RiskLevel::High,
                description: format!("工具 '{}' 被策略禁止", tool_name),
                auto_confirm_timeout_secs: None,
            };
        }

        let risk = self.assess_risk(tool_name, args);

        let level = if self.auto_approve_tools.contains(tool_name) {
            ConfirmationLevel::Auto
        } else if self.confirm_tools.contains(tool_name) || risk == RiskLevel::High {
            ConfirmationLevel::Confirm
        } else if risk == RiskLevel::Medium {
            ConfirmationLevel::Notify
        } else {
            ConfirmationLevel::Auto
        };

        let description = match level {
            ConfirmationLevel::Auto => format!("工具 '{}' 自动批准 (风险: {:?})", tool_name, risk),
            ConfirmationLevel::Notify => format!("工具 '{}' 需要通知用户 (风险: {:?})", tool_name, risk),
            ConfirmationLevel::Confirm => format!("工具 '{}' 需要用户确认 (风险: {:?})", tool_name, risk),
        };

        ConfirmationRequest {
            tool_name: tool_name.to_string(),
            args: args.clone(),
            risk_level: risk,
            description,
            auto_confirm_timeout_secs: if level == ConfirmationLevel::Notify { Some(5) } else { None },
        }
    }

    fn assess_risk(&self, tool_name: &str, args: &Value) -> RiskLevel {
        if self.is_delete_operation(tool_name, args) {
            return RiskLevel::High;
        }
        if self.is_write_operation(tool_name, args) {
            return RiskLevel::Medium;
        }
        RiskLevel::Low
    }

    fn is_delete_operation(&self, tool_name: &str, args: &Value) -> bool {
        let lower = tool_name.to_lowercase();
        for cmd in &self.dangerous_commands {
            if lower.contains(cmd) {
                return true;
            }
        }
        if let Some(args_str) = args.as_str() {
            let lower = args_str.to_lowercase();
            for cmd in &self.dangerous_commands {
                if lower.contains(cmd) {
                    return true;
                }
            }
        }
        if let Some(args_obj) = args.as_object() {
            if let Some(command) = args_obj.get("command").and_then(|c| c.as_str()) {
                let lower = command.to_lowercase();
                for cmd in &self.dangerous_commands {
                    if lower.contains(cmd) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_write_operation(&self, tool_name: &str, _args: &Value) -> bool {
        let lower = tool_name.to_lowercase();
        lower.contains("add") || lower.contains("update") || lower.contains("create") || lower.contains("write")
    }

    pub fn should_auto_approve(&self, request: &ConfirmationRequest) -> bool {
        request.risk_level == RiskLevel::Low
            && !self.confirm_tools.contains(&request.tool_name)
    }

    pub fn should_deny(&self, request: &ConfirmationRequest) -> bool {
        self.deny_tools.contains(&request.tool_name)
    }
}

impl Default for HitlPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_auto_approve_read() {
        let policy = HitlPolicy::new();
        let req = policy.check("i_rs", &json!({"command": "list", "tool": "weight"}));
        assert!(policy.should_auto_approve(&req));
    }

    #[test]
    fn test_confirm_delete() {
        let policy = HitlPolicy::new();
        let req = policy.check("i_rs", &json!({"command": "delete", "tool": "weight"}));
        assert_eq!(req.risk_level, RiskLevel::High);
        assert!(!policy.should_auto_approve(&req));
    }

    #[test]
    fn test_deny_tool() {
        let policy = HitlPolicy::new().deny("dangerous_tool");
        let req = policy.check("dangerous_tool", &json!({}));
        assert!(policy.should_deny(&req));
    }

    #[test]
    fn test_explicit_confirm() {
        let policy = HitlPolicy::new().require_confirm("i_rs");
        let req = policy.check("i_rs", &json!({"command": "add"}));
        assert!(!policy.should_auto_approve(&req));
    }

    #[test]
    fn test_explicit_auto_approve() {
        let policy = HitlPolicy::new().auto_approve("i_rs");
        let req = policy.check("i_rs", &json!({"command": "list", "tool": "weight"}));
        assert!(policy.should_auto_approve(&req));
    }
}
