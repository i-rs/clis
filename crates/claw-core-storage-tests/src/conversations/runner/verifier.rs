use super::{ScriptStep, ToolCallInfo};

/// Compare actual tool call against expected values.
/// Returns list of deviation descriptions (empty = perfect match).
pub fn check_tool_call(step: &ScriptStep, actual: &ToolCallInfo) -> Vec<String> {
    let mut deviations = Vec::new();

    if let Some(ref expected_tool) = step.expected_tool
        && &actual.tool != expected_tool
    {
        deviations.push(format!(
            "工具不匹配: 期望={}, 实际={}",
            expected_tool, actual.tool
        ));
    }

    if let Some(ref expected_cmd) = step.expected_command {
        match &actual.command {
            Some(cmd) if cmd != expected_cmd => {
                deviations.push(format!("命令不匹配: 期望={}, 实际={}", expected_cmd, cmd));
            }
            None => {
                deviations.push(format!("未调用命令: 期望={}", expected_cmd));
            }
            _ => {}
        }
    }

    if let Some(ref expected_args) = step.expected_args {
        for (key, expected_val) in expected_args {
            match actual.args.get(key) {
                Some(actual_val) if actual_val != expected_val => {
                    deviations.push(format!(
                        "参数 {} 不匹配: 期望={}, 实际={}",
                        key, expected_val, actual_val
                    ));
                }
                None => {
                    deviations.push(format!("缺少参数: {}={}", key, expected_val));
                }
                _ => {}
            }
        }
    }

    deviations
}

/// Check reply text contains expected keywords.
/// Returns list of missing keywords (empty = all found).
pub fn check_reply(step: &ScriptStep, reply: &str) -> Vec<String> {
    let mut missing = Vec::new();
    if let Some(ref check) = step.check_reply {
        for keyword in &check.contains {
            if !reply.contains(keyword) {
                missing.push(keyword.clone());
            }
        }
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_tool_call_perfect_match() {
        let step = ScriptStep {
            step: 1,
            title: "test".into(),
            user_message: "hello".into(),
            expected_tool: Some("i-rs-kv".into()),
            expected_command: Some("add".into()),
            expected_args: {
                let mut m = HashMap::new();
                m.insert("KEY".into(), "blog_url".into());
                Some(m)
            },
            expected_flags: None,
            check_reply: None,
            verify_storage: None,
        };
        let actual = ToolCallInfo::new("i-rs-kv")
            .with_command("add")
            .with_arg("KEY", "blog_url");

        let deviations = check_tool_call(&step, &actual);
        assert!(
            deviations.is_empty(),
            "expected no deviations, got: {:?}",
            deviations
        );
    }

    #[test]
    fn test_tool_call_mismatch() {
        let step = ScriptStep {
            step: 1,
            title: "test".into(),
            user_message: "hello".into(),
            expected_tool: Some("i-rs-kv".into()),
            expected_command: Some("add".into()),
            expected_args: {
                let mut m = HashMap::new();
                m.insert("KEY".into(), "blog_url".into());
                Some(m)
            },
            expected_flags: None,
            check_reply: None,
            verify_storage: None,
        };
        let actual = ToolCallInfo::new("i-rs-weight").with_command("list");

        let deviations = check_tool_call(&step, &actual);
        assert!(!deviations.is_empty(), "expected deviations");
        assert!(deviations.iter().any(|d| d.contains("工具不匹配")));
    }

    #[test]
    fn test_reply_check_contains() {
        let step = ScriptStep {
            step: 1,
            title: "test".into(),
            user_message: "hello".into(),
            expected_tool: None,
            expected_command: None,
            expected_args: None,
            expected_flags: None,
            check_reply: Some(crate::conversations::runner::ReplyCheck {
                contains: vec!["已记录".into(), "blog_url".into()],
            }),
            verify_storage: None,
        };
        let missing = check_reply(&step, "已记录 blog_url value=example.com");
        assert!(
            missing.is_empty(),
            "expected no missing, got: {:?}",
            missing
        );

        let missing2 = check_reply(&step, "出错了");
        assert_eq!(missing2.len(), 2);
    }
}
