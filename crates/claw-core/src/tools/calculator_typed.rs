use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ClawError;
use crate::tools::{ToolContext, TypedClawTool, TypedToolAdapter};

#[derive(Serialize, Deserialize)]
pub struct CalculatorOutput {
    pub expression: String,
    pub result: f64,
    pub formatted: String,
}

pub struct CalculatorTyped;

impl CalculatorTyped {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl Default for CalculatorTyped {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorTyped {
    pub fn as_claw_tool() -> TypedToolAdapter {
        TypedToolAdapter::new(Box::new(Self))
    }
}

#[async_trait::async_trait]
impl TypedClawTool for CalculatorTyped {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate a safe arithmetic expression. \
         Supports +, -, *, /, %, ^ (power), parentheses, and unary minus. \
         Use when you need to compute numbers — reduce hallucination risk by \
         delegating calculations to this tool."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Arithmetic expression to evaluate, e.g. '2 + 3 * 4', '(100 - 20) / 4', '2^10'"
                }
            },
            "required": ["expression"],
            "additionalProperties": false
        })
    }

    fn output_schema(&self) -> Option<Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "expression": { "type": "string" },
                "result": { "type": "number" },
                "formatted": { "type": "string" }
            }
        }))
    }

    async fn execute_typed(&self, args: &Value, _ctx: &ToolContext) -> Result<Value, ClawError> {
        let expr = args
            .get("expression")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if expr.is_empty() {
            return Err(ClawError::Validation(
                "Please provide an expression".to_string(),
            ));
        }

        let result = super::calculator::eval(expr)
            .map_err(|msg| ClawError::Execution(format!("计算错误: {}", msg)))?;

        let formatted = if result.fract() == 0.0 && result < 1e15 {
            format!("{}", result as i64)
        } else {
            format!("{}", result)
        };

        let output = CalculatorOutput {
            expression: expr.to_string(),
            result,
            formatted: format!("{} = {}", expr, formatted),
        };

        serde_json::to_value(&output)
            .map_err(|e| ClawError::Execution(format!("序列化失败: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::shared_client;
    use crate::test_helpers::test_config;
    use crate::tools::ToolContext;
    use serde_json::json;

    fn test_ctx() -> ToolContext {
        ToolContext {
            config: test_config(),
            http_client: shared_client(),
            delegate_runtime: None,
            user_id: "test".to_string(),
        }
    }

    #[tokio::test]
    async fn test_basic_arithmetic_typed() {
        let tool = CalculatorTyped;
        let ctx = test_ctx();

        let result = tool
            .execute_typed(&json!({ "expression": "2 + 3" }), &ctx)
            .await
            .unwrap();
        let output: CalculatorOutput = serde_json::from_value(result).unwrap();
        assert_eq!(output.result, 5.0);
        assert_eq!(output.formatted, "2 + 3 = 5");
    }

    #[tokio::test]
    async fn test_precedence_typed() {
        let tool = CalculatorTyped;
        let ctx = test_ctx();

        let result = tool
            .execute_typed(&json!({ "expression": "2 + 3 * 4" }), &ctx)
            .await
            .unwrap();
        let output: CalculatorOutput = serde_json::from_value(result).unwrap();
        assert_eq!(output.result, 14.0);
    }

    #[tokio::test]
    async fn test_error_empty() {
        let tool = CalculatorTyped;
        let ctx = test_ctx();

        let result = tool.execute_typed(&json!({ "expression": "" }), &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_error_division_by_zero() {
        let tool = CalculatorTyped;
        let ctx = test_ctx();

        let result = tool
            .execute_typed(&json!({ "expression": "10 / 0" }), &ctx)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_output_schema() {
        let tool = CalculatorTyped;
        let schema = tool.output_schema().unwrap();
        assert_eq!(schema["properties"]["expression"]["type"], "string");
        assert_eq!(schema["properties"]["result"]["type"], "number");
        assert_eq!(schema["properties"]["formatted"]["type"], "string");
    }
}
