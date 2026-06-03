use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    pub tool_name: String,
    pub args_template: Value,
    pub output_key: String,
    pub condition: Option<ChainCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainCondition {
    pub field: String,
    pub operator: ConditionOp,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ConditionOp {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    IsEmpty,
    IsNotEmpty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChain {
    pub name: String,
    pub description: String,
    pub steps: Vec<ChainStep>,
}

impl ToolChain {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            steps: Vec::new(),
        }
    }

    pub fn then(mut self, tool_name: &str, output_key: &str, args_template: Value) -> Self {
        self.steps.push(ChainStep {
            tool_name: tool_name.to_string(),
            args_template,
            output_key: output_key.to_string(),
            condition: None,
        });
        self
    }

    #[allow(dead_code)]
    pub fn then_if(
        mut self,
        tool_name: &str,
        output_key: &str,
        args_template: Value,
        condition: ChainCondition,
    ) -> Self {
        self.steps.push(ChainStep {
            tool_name: tool_name.to_string(),
            args_template,
            output_key: output_key.to_string(),
            condition: Some(condition),
        });
        self
    }

    #[allow(dead_code)]
    pub fn resolve_args(
        template: &Value,
        context: &ChainContext,
    ) -> Value {
        match template {
            Value::String(s) => {
                let mut result = s.clone();
                for (key, val) in &context.outputs {
                    let placeholder = format!("{{{{{}}}}}", key);
                    let val_str = match val {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    result = result.replace(&placeholder, &val_str);
                }
                Value::String(result)
            }
            Value::Object(map) => {
                let resolved: serde_json::Map<String, Value> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::resolve_args(v, context)))
                    .collect();
                Value::Object(resolved)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| Self::resolve_args(v, context)).collect())
            }
            other => other.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn check_condition(condition: &Option<ChainCondition>, context: &ChainContext) -> bool {
        let Some(cond) = condition else {
            return true;
        };
        let field_val = context.get_value(&cond.field);
        match cond.operator {
            ConditionOp::Equals => field_val == Some(&cond.value),
            ConditionOp::NotEquals => field_val != Some(&cond.value),
            ConditionOp::Contains => {
                if let Some(Value::String(s)) = field_val
                    && let Value::String(needle) = &cond.value
                {
                    return s.contains(needle.as_str());
                }
                false
            }
            ConditionOp::GreaterThan => {
                if let (Some(Value::Number(a)), Value::Number(b)) = (field_val, &cond.value) {
                    a.as_f64().zip(b.as_f64()).map(|(a, b)| a > b).unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionOp::LessThan => {
                if let (Some(Value::Number(a)), Value::Number(b)) = (field_val, &cond.value) {
                    a.as_f64().zip(b.as_f64()).map(|(a, b)| a < b).unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionOp::IsEmpty => {
                field_val.is_none_or(|v| v.is_null() || v == &Value::String(String::new()))
            }
            ConditionOp::IsNotEmpty => {
                field_val.is_some_and(|v| !v.is_null() && v != &Value::String(String::new()))
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ChainContext {
    pub outputs: std::collections::HashMap<String, Value>,
    pub initial_input: Value,
}

#[allow(dead_code)]
impl ChainContext {
    pub fn new(initial_input: Value) -> Self {
        Self {
            outputs: std::collections::HashMap::new(),
            initial_input,
        }
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.outputs.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.outputs.get(key)
    }

    pub fn get_value(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() == 1 {
            return self.outputs.get(path);
        }
        let mut current = self.outputs.get(parts[0])?;
        for part in &parts[1..] {
            current = current.get(*part)?;
        }
        Some(current)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.outputs.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ChainResult {
    pub chain_name: String,
    pub steps_completed: usize,
    pub steps_total: usize,
    pub final_output: Value,
    pub intermediate_outputs: std::collections::HashMap<String, Value>,
}

#[allow(dead_code)]
impl ChainResult {
    pub fn is_complete(&self) -> bool {
        self.steps_completed == self.steps_total
    }
}

pub fn builtin_chains() -> Vec<ToolChain> {
    vec![
        ToolChain::new(
            "record_and_stats",
            "记录数据后查看统计",
        )
        .then("i_rs", "record_result", serde_json::json!({
            "command": "add",
            "tool": "{{tool}}",
            "args": ["{{value}}"]
        }))
        .then("i_rs", "stats_result", serde_json::json!({
            "command": "stats",
            "tool": "{{tool}}"
        })),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_builder() {
        let chain = ToolChain::new("test", "test chain")
            .then("tool_a", "output_a", serde_json::json!({"key": "value"}))
            .then("tool_b", "output_b", serde_json::json!({"input": "{{output_a}}"}));
        assert_eq!(chain.steps.len(), 2);
    }

    #[test]
    fn test_resolve_args() {
        let mut ctx = ChainContext::new(serde_json::json!(null));
        ctx.set("name", serde_json::json!("体重"));
        let resolved = ToolChain::resolve_args(
            &serde_json::json!({"tool": "{{name}}"}),
            &ctx,
        );
        assert_eq!(resolved["tool"], "体重");
    }

    #[test]
    fn test_resolve_nested_args() {
        let mut ctx = ChainContext::new(serde_json::json!(null));
        ctx.set("value", serde_json::json!(70));
        let resolved = ToolChain::resolve_args(
            &serde_json::json!({"args": ["{{value}}"]}),
            &ctx,
        );
        assert_eq!(resolved["args"][0], "70");
    }

    #[test]
    fn test_condition_equals() {
        let mut ctx = ChainContext::new(serde_json::json!(null));
        ctx.set("status", serde_json::json!("ok"));
        let cond = ChainCondition {
            field: "status".to_string(),
            operator: ConditionOp::Equals,
            value: serde_json::json!("ok"),
        };
        assert!(ToolChain::check_condition(&Some(cond), &ctx));
    }

    #[test]
    fn test_condition_not_met() {
        let ctx = ChainContext::new(serde_json::json!(null));
        let cond = ChainCondition {
            field: "status".to_string(),
            operator: ConditionOp::Equals,
            value: serde_json::json!("ok"),
        };
        assert!(!ToolChain::check_condition(&Some(cond), &ctx));
    }

    #[test]
    fn test_condition_no_condition() {
        let ctx = ChainContext::new(serde_json::json!(null));
        assert!(ToolChain::check_condition(&None, &ctx));
    }

    #[test]
    fn test_chain_context() {
        let mut ctx = ChainContext::new(serde_json::json!("input"));
        ctx.set("result", serde_json::json!({"count": 5}));
        assert_eq!(ctx.get_value("result.count").and_then(|v| v.as_i64()), Some(5));
    }

    #[test]
    fn test_chain_result() {
        let result = ChainResult {
            chain_name: "test".to_string(),
            steps_completed: 2,
            steps_total: 2,
            final_output: serde_json::json!("done"),
            intermediate_outputs: std::collections::HashMap::new(),
        };
        assert!(result.is_complete());
    }

    #[test]
    fn test_builtin_chains() {
        let chains = builtin_chains();
        assert!(!chains.is_empty());
        assert_eq!(chains[0].name, "record_and_stats");
    }
}
