use crate::error::ClawError;
use serde_json::Value;

use super::chart_render;
use super::{ClawTool, ToolContext};

/// Chart tool: generates ASCII bar charts and line charts from i-rs CLI data.
pub struct ChartTool;

#[async_trait::async_trait]
impl ClawTool for ChartTool {
    fn name(&self) -> &str {
        "chart"
    }

    fn description(&self) -> &str {
        "Generate ASCII bar charts and line charts from i-rs CLI data. Useful for visualizing trends in weight, mood, sleep, exercise, budget, etc."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "tool": {
                    "type": "string",
                    "description": "i-rs CLI tool name to get data from (e.g., weight, mood, sleep)",
                    "enum": _enabled_cli_tools
                },
                "command": {
                    "type": "string",
                    "description": "Command to run (usually 'list' or 'stats')",
                    "default": "list"
                },
                "chart_type": {
                    "type": "string",
                    "description": "Type of chart: 'bar' for vertical bars, 'line' for line chart",
                    "enum": ["bar", "line"],
                    "default": "bar"
                },
                "value_field": {
                    "type": "string",
                    "description": "Field name containing the numeric value to chart (e.g., 'value', 'weight', 'hours', 'amount')",
                    "default": "value"
                },
                "label_field": {
                    "type": "string",
                    "description": "Field name for the X-axis label (e.g., 'name', 'date', 'day')",
                    "default": "name"
                },
                "width": {
                    "type": "integer",
                    "description": "Width of the chart in characters (default: 40, max: 80)",
                    "default": 40,
                    "minimum": 20,
                    "maximum": 80
                },
                "height": {
                    "type": "integer",
                    "description": "Height of bar/line chart in characters (default: 10, max: 20)",
                    "default": 10,
                    "minimum": 5,
                    "maximum": 20
                },
                "extra_args": {
                    "type": "string",
                    "description": "Extra CLI arguments to pass to the tool (e.g., '--days 30')",
                    "default": ""
                }
            },
            "required": ["tool"]
        })
    }

    async fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        let tool = args
            .get("tool")
            .and_then(|v| v.as_str())
            .ok_or("缺少必要参数: tool")?;
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("list");
        let chart_type = args
            .get("chart_type")
            .and_then(|v| v.as_str())
            .unwrap_or("bar");
        let value_field = args
            .get("value_field")
            .and_then(|v| v.as_str())
            .unwrap_or("value");
        let label_field = args
            .get("label_field")
            .and_then(|v| v.as_str())
            .unwrap_or("name");
        let width = args
            .get("width")
            .and_then(|v| v.as_u64())
            .unwrap_or(40)
            .clamp(20, 80) as usize;
        let height = args
            .get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(10)
            .clamp(5, 20) as usize;
        let extra_args = args
            .get("extra_args")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let extra: Vec<&str> = if extra_args.is_empty() {
            vec![]
        } else {
            extra_args.split_whitespace().collect()
        };

        let tool_owned = tool.to_string();
        let command_owned = command.to_string();
        let extra_owned: Vec<String> = extra.iter().map(|s| s.to_string()).collect();
        let json_data = tokio::task::spawn_blocking(move || {
            crate::utils::run_i_rs_json(
                &tool_owned,
                &command_owned,
                &extra_owned.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                30,
            )
        })
        .await
        .map_err(|e| ClawError::Execution(format!("图表工具任务失败: {}", e)))?
        .map_err(ClawError::Execution)?;

        let data_points = extract_data_points(&json_data, label_field, value_field)?;

        if data_points.is_empty() {
            return Ok("没有足够的数据来生成图表".to_string());
        }

        match chart_type {
            "line" => Ok(chart_render::generate_line_chart(
                &data_points,
                width,
                height,
            )),
            _ => Ok(chart_render::generate_bar_chart(
                &data_points,
                width,
                height,
            )),
        }
    }
}

/// Extract data points from JSON CLI result.
/// Supports: { data: [{...}, ...] } and { data: { key: value, ... } } formats.
fn extract_data_points(
    json: &Value,
    label_field: &str,
    value_field: &str,
) -> Result<Vec<chart_render::DataPoint>, String> {
    let mut points = Vec::new();

    // Format 1: { data: [...] }
    if let Some(data) = json.get("data") {
        if let Some(arr) = data.as_array() {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    let label = obj
                        .get(label_field)
                        .and_then(|v| v.as_str())
                        .unwrap_or("?")
                        .to_string();
                    let value = parse_numeric(obj, value_field);
                    if let Some(v) = value {
                        points.push(chart_render::DataPoint { label, value: v });
                    }
                }
            }
            if !points.is_empty() {
                return Ok(points);
            }
        }

        // Format 2: { data: { key: value, ... } } (e.g., stats)
        if let Some(obj) = data.as_object() {
            for (key, val) in obj {
                if let Some(n) = val.as_f64().or_else(|| val.as_i64().map(|i| i as f64)) {
                    points.push(chart_render::DataPoint {
                        label: key.clone(),
                        value: n,
                    });
                }
            }
            if !points.is_empty() {
                return Ok(points);
            }
        }
    }

    // Format 3: flat array at top level
    if let Some(arr) = json.as_array() {
        for item in arr {
            if let Some(obj) = item.as_object() {
                let label = obj
                    .get(label_field)
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string();
                let value = parse_numeric(obj, value_field);
                if let Some(v) = value {
                    points.push(chart_render::DataPoint { label, value: v });
                }
            }
        }
        if !points.is_empty() {
            return Ok(points);
        }
    }

    // Format 4: map at top level { key: value, ... }
    if let Some(obj) = json.as_object() {
        for (key, val) in obj {
            if let Some(n) = val.as_f64().or_else(|| val.as_i64().map(|i| i as f64)) {
                points.push(chart_render::DataPoint {
                    label: key.clone(),
                    value: n,
                });
            }
        }
        if !points.is_empty() {
            return Ok(points);
        }
    }

    Err(format!(
        "未能从 JSON 中找到可用的数据字段 (label='{}', value='{}')",
        label_field, value_field
    ))
}

/// Try to parse a numeric value from an object by field name (or "value" fallback).
fn parse_numeric(obj: &serde_json::Map<String, Value>, field: &str) -> Option<f64> {
    if let Some(v) = obj.get(field) {
        if let Some(n) = v.as_f64() {
            return Some(n);
        }
        if let Some(n) = v.as_i64() {
            return Some(n as f64);
        }
        if let Some(s) = v.as_str()
            && let Ok(n) = s.parse::<f64>()
        {
            return Some(n);
        }
    }

    for &f in &[
        "value", "amount", "count", "total", "hours", "minutes", "days", "score", "price",
    ] {
        if f == field {
            continue;
        }
        if let Some(v) = obj.get(f) {
            if let Some(n) = v.as_f64() {
                return Some(n);
            }
            if let Some(n) = v.as_i64() {
                return Some(n as f64);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_data_array() {
        let json = serde_json::json!({
            "data": [
                {"name": "Mon", "value": 10},
                {"name": "Tue", "value": 20},
            ]
        });
        let points = extract_data_points(&json, "name", "value").unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].label, "Mon");
        assert_eq!(points[0].value, 10.0);
        assert_eq!(points[1].label, "Tue");
        assert_eq!(points[1].value, 20.0);
    }

    #[test]
    fn test_extract_from_data_object() {
        let json = serde_json::json!({
            "data": {"Mon": 10, "Tue": 20}
        });
        let points = extract_data_points(&json, "name", "value").unwrap();
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn test_extract_from_flat_array() {
        let json = serde_json::json!([
            {"date": "2024-01", "weight": 70.5},
            {"date": "2024-02", "weight": 69.8},
        ]);
        let points = extract_data_points(&json, "date", "weight").unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].value, 70.5);
    }

    #[test]
    fn test_extract_from_flat_map() {
        let json = serde_json::json!({"Mon": 10, "Tue": 20});
        let points = extract_data_points(&json, "name", "value").unwrap();
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn test_extract_empty_data() {
        let json = serde_json::json!({"data": []});
        let result = extract_data_points(&json, "name", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_custom_fields() {
        let json = serde_json::json!({
            "data": [
                {"day": "Mon", "amount": 100},
                {"day": "Tue", "amount": 200},
            ]
        });
        let points = extract_data_points(&json, "day", "amount").unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].label, "Mon");
        assert_eq!(points[0].value, 100.0);
    }

    #[test]
    fn test_extract_string_numeric() {
        let json = serde_json::json!({
            "data": [{"name": "A", "value": "42.5"}]
        });
        let points = extract_data_points(&json, "name", "value").unwrap();
        assert_eq!(points[0].value, 42.5);
    }
}
