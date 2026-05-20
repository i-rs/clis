use crate::error::ClawError;
use serde_json::Value;

use super::{ClawTool, ToolContext};

/// Chart tool: generates ASCII bar charts and line charts from i-rs CLI data.
pub struct ChartTool;

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

    fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
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

        // Execute the i-rs CLI command
        let json_data = run_i_rs_cli_json(tool, command, extra_args)?;

        // Extract numeric data from JSON result
        let data_points = extract_data_points(&json_data, label_field, value_field)?;

        if data_points.is_empty() {
            return Ok("没有足够的数据来生成图表".to_string());
        }

        match chart_type {
            "line" => Ok(generate_line_chart(&data_points, width, height)),
            _ => Ok(generate_bar_chart(&data_points, width, height)),
        }
    }
}

/// A labeled numeric data point.
struct DataPoint {
    label: String,
    value: f64,
}

/// Run an i-rs CLI command and parse JSON output.
fn run_i_rs_cli_json(tool: &str, command: &str, extra_args: &str) -> Result<Value, ClawError> {
    let mut cmd = std::process::Command::new("i-rs");
    cmd.arg(tool).arg(command).arg("--json");

    if !extra_args.is_empty() {
        for arg in extra_args.split_whitespace() {
            cmd.arg(arg);
        }
    }

    let output = cmd
        .output()
        .map_err(|e| ClawError::Execution(format!("执行 i-rs {} {} 失败: {}", tool, command, e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ClawError::Execution(format!(
            "i-rs {} {} 返回错误: {}",
            tool,
            command,
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    serde_json::from_str(&stdout).map_err(|e| ClawError::Execution(format!("解析 JSON 输出失败: {}", e)))
}

/// Extract data points from JSON CLI result.
/// Supports: { data: [{...}, ...] } and { data: { key: value, ... } } formats.
fn extract_data_points(
    json: &Value,
    label_field: &str,
    value_field: &str,
) -> Result<Vec<DataPoint>, String> {
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
                        points.push(DataPoint { label, value: v });
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
                    points.push(DataPoint {
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
                    points.push(DataPoint { label, value: v });
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
                points.push(DataPoint {
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
    // Try exact field first
    if let Some(v) = obj.get(field) {
        if let Some(n) = v.as_f64() {
            return Some(n);
        }
        if let Some(n) = v.as_i64() {
            return Some(n as f64);
        }
        if let Some(s) = v.as_str()
            && let Ok(n) = s.parse::<f64>() {
                return Some(n);
            }
    }

    // Try common numeric field names
    for &f in &["value", "amount", "count", "total", "hours", "minutes", "days", "score", "price"] {
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

/// Generate an ASCII bar chart.
fn generate_bar_chart(data: &[DataPoint], width: usize, height: usize) -> String {
    if data.is_empty() {
        return "(无数据)".to_string();
    }

    let max_val = data
        .iter()
        .map(|d| d.value)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(1.0)
        .max(1.0);

    let chart_width = width.min(80);
    let chart_height = height.clamp(5, 20);
    let bar_width = (chart_width.saturating_sub(6)).max(5) / data.len().max(1);
    let bar_width = bar_width.clamp(1, 10);

    let mut output = String::new();
    output.push_str(&format!(
        "┌ {} 柱状图 (max={:.1})\n",
        data.first().map(|d| &d.label).unwrap_or(&"?".to_string()),
        max_val
    ));

    // Top border
    let top_w = bar_width * data.len() + 2;
    output.push('┌');
    for _ in 0..top_w {
        output.push('─');
    }
    output.push_str("┐\n");

    for row in 0..chart_height {
        let threshold = max_val * (chart_height - row) as f64 / chart_height as f64;
        output.push('│');
        for point in data {
            let bar_char = if point.value >= threshold {
                '█'
            } else if point.value >= threshold * 0.7 {
                '▓'
            } else if point.value >= threshold * 0.4 {
                '▒'
            } else if point.value >= threshold * 0.15 {
                '░'
            } else {
                ' '
            };
            for _ in 0..bar_width {
                output.push(bar_char);
            }
        }
        // Y-axis label
        if row == 0 {
            output.push_str(&format!("│ {:.0}", max_val));
        } else if row == chart_height - 1 {
            output.push_str("│ 0");
        }
        output.push('\n');
    }

    // Bottom border
    output.push('└');
    for _ in 0..top_w {
        output.push('─');
    }
    output.push_str("┘\n");

    // X-axis labels
    output.push(' ');
    for point in data {
        let label = if point.label.len() > bar_width {
            &point.label[..bar_width]
        } else {
            &point.label
        };
        output.push_str(&format!("{:^width$}", label, width = bar_width));
    }
    output.push('\n');

    output
}

/// Generate an ASCII line chart.
fn generate_line_chart(data: &[DataPoint], width: usize, height: usize) -> String {
    if data.len() < 2 {
        return "(至少需要2个数据点来绘制折线图)".to_string();
    }

    let max_val = data
        .iter()
        .map(|d| d.value)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(1.0)
        .max(1.0);
    let min_val = data
        .iter()
        .map(|d| d.value)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0)
        .min(0.0);
    let range = (max_val - min_val).max(1.0);

    let chart_height = height.clamp(5, 20);
    let chart_width = width.min(80);

    let mut output = String::new();
    output.push_str(&format!(
        "┌ 折线图 (min={:.1}, max={:.1})\n",
        min_val, max_val
    ));

    // Build the grid
    let grid_cols = data.len();
    let col_width = ((chart_width.saturating_sub(6)) / grid_cols.max(1)).max(1);

    // Top border
    let top_w = col_width * grid_cols + 2;
    output.push('┌');
    for _ in 0..top_w {
        output.push('─');
    }
    output.push_str("┐\n");

    for row in 0..chart_height {
        let threshold = max_val - (range * row as f64 / chart_height as f64);
        output.push('│');
        for (i, point) in data.iter().enumerate() {
            let is_point = (point.value - min_val) >= (range * (chart_height - 1 - row) as f64 / chart_height as f64)
                && (point.value - min_val) < (range * (chart_height - row) as f64 / chart_height as f64);

            // Check if there's a line to previous/next point
            let connects_left = i > 0
                && is_between(
                    data[i - 1].value,
                    point.value,
                    threshold,
                    range / chart_height as f64,
                );
            let connects_right = i + 1 < data.len()
                && is_between(
                    data[i + 1].value,
                    point.value,
                    threshold,
                    range / chart_height as f64,
                );

            let c = if is_point {
                if connects_left && connects_right {
                    '┼'
                } else if connects_left {
                    '╰'
                } else if connects_right {
                    '╭'
                } else {
                    '●'
                }
            } else if connects_left && connects_right {
                '─'
            } else if connects_left {
                '╯'
            } else if connects_right {
                '╮'
            } else {
                ' '
            };
            output.push(c);
            for _ in 1..col_width {
                if is_point {
                    output.push(' ');
                } else if connects_left || connects_right {
                    output.push('─');
                } else {
                    output.push(' ');
                }
            }
        }
        // Y-axis label
        if row == 0 {
            output.push_str(&format!("│ {:.0}", max_val));
        } else if row == chart_height - 1 {
            output.push_str(&format!("│ {:.0}", min_val));
        }
        output.push('\n');
    }

    // Bottom border
    output.push('└');
    for _ in 0..top_w {
        output.push('─');
    }
    output.push_str("┘\n");

    // X-axis labels
    output.push(' ');
    for point in data {
        let label = if point.label.len() > col_width {
            &point.label[..col_width]
        } else {
            &point.label
        };
        output.push_str(&format!("{:^width$}", label, width = col_width));
    }
    output.push('\n');

    output
}

/// Check if a value is within the threshold row of a line between two points.
fn is_between(a: f64, b: f64, threshold: f64, epsilon: f64) -> bool {
    let min_v = a.min(b);
    let max_v = a.max(b);
    threshold >= min_v - epsilon && threshold <= max_v + epsilon
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_between ──

    #[test]
    fn test_is_between_direct() {
        assert!(is_between(0.0, 10.0, 5.0, 1.0));
    }

    #[test]
    fn test_is_between_at_edge() {
        // threshold within epsilon of min
        assert!(is_between(5.0, 10.0, 4.5, 1.0));
        // threshold within epsilon of max
        assert!(is_between(5.0, 10.0, 10.5, 1.0));
    }

    #[test]
    fn test_is_between_outside() {
        assert!(!is_between(10.0, 20.0, 5.0, 1.0));
        assert!(!is_between(10.0, 20.0, 25.0, 1.0));
    }

    #[test]
    fn test_is_between_negative() {
        assert!(is_between(-10.0, 0.0, -5.0, 1.0));
        assert!(!is_between(-10.0, 0.0, -15.0, 1.0));
    }

    #[test]
    fn test_is_between_zero_range() {
        assert!(is_between(5.0, 5.0, 5.0, 0.1));
        assert!(!is_between(5.0, 5.0, 5.5, 0.1));
    }

    // ── extract_data_points ──

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

    // ── generate_bar_chart ──

    #[test]
    fn test_bar_chart_empty() {
        let result = generate_bar_chart(&[], 40, 10);
        assert_eq!(result, "(无数据)");
    }

    #[test]
    fn test_bar_chart_single_point() {
        let data = vec![DataPoint { label: "Test".into(), value: 100.0 }];
        let result = generate_bar_chart(&data, 30, 5);
        assert!(result.contains("Test"), "应包含标签");
        assert!(result.contains("█"), "应包含柱状条");
    }

    #[test]
    fn test_bar_chart_multiple_points() {
        let data = vec![
            DataPoint { label: "A".into(), value: 50.0 },
            DataPoint { label: "B".into(), value: 100.0 },
            DataPoint { label: "C".into(), value: 30.0 },
        ];
        let result = generate_bar_chart(&data, 40, 10);
        assert!(result.contains("A"), "应包含标签 A");
        assert!(result.contains("B"), "应包含标签 B");
        assert!(result.contains("C"), "应包含标签 C");
        assert!(result.contains("█"), "应包含柱状条");
        assert!(result.contains("100"), "应包含最大值标注");
    }

    // ── generate_line_chart ──

    #[test]
    fn test_line_chart_empty() {
        let data = vec![DataPoint { label: "A".into(), value: 10.0 }];
        let result = generate_line_chart(&data, 40, 10);
        assert!(result.contains("至少需要2个数据点"));
    }

    #[test]
    fn test_line_chart_valid() {
        let data = vec![
            DataPoint { label: "Mon".into(), value: 10.0 },
            DataPoint { label: "Tue".into(), value: 20.0 },
            DataPoint { label: "Wed".into(), value: 15.0 },
        ];
        let result = generate_line_chart(&data, 40, 10);
        assert!(result.contains("Mon"), "应包含标签");
        assert!(result.contains("折线图"), "应包含标题");
        assert!(result.contains("20"), "应包含最大值");
        assert!(result.contains("0"), "应包含最小值");
    }

    #[test]
    fn test_line_chart_flat_line() {
        let data = vec![
            DataPoint { label: "A".into(), value: 50.0 },
            DataPoint { label: "B".into(), value: 50.0 },
        ];
        let result = generate_line_chart(&data, 40, 10);
        // 直线情况下所有点都在同一行
        assert!(result.contains("50"));
    }

    #[test]
    fn test_line_chart_differences() {
        let data = vec![
            DataPoint { label: "min".into(), value: 0.0 },
            DataPoint { label: "mid".into(), value: 50.0 },
            DataPoint { label: "max".into(), value: 100.0 },
        ];
        let result = generate_line_chart(&data, 40, 10);
        assert!(result.contains("0"));
        assert!(result.contains("100"));
        // 确认不同高度的点
        let line_count = result.lines().count();
        assert!(line_count > 5, "图表应有多行");
    }
}
