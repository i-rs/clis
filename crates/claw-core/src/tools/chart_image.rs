use crate::error::ClawError;
use crate::providers::image_gen::{self, ChartDataPoint};
use serde_json::Value;

use super::{ClawTool, ToolContext};

pub struct ChartImageTool;

#[async_trait::async_trait]
impl ClawTool for ChartImageTool {
    fn name(&self) -> &str {
        "chart_image"
    }

    fn description(&self) -> &str {
        "Generate a chart as an image (SVG or PNG) from structured data. \
         Supported chart types: bar, line, pie, radar. \
         The image is saved to the configured images directory and can be \
         displayed in the Dashboard. \
         Use this to visualize data trends from i-rs tools."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "chart_type": {
                    "type": "string",
                    "description": "Chart type: 'bar', 'line', 'pie', or 'radar'",
                    "enum": ["bar", "line", "pie", "radar"],
                    "default": "bar"
                },
                "data": {
                    "type": "array",
                    "description": "Array of {label, value} data points to chart",
                    "items": {
                        "type": "object",
                        "properties": {
                            "label": {"type": "string", "description": "Data label (X-axis or slice name)"},
                            "value": {"type": "number", "description": "Numeric value"}
                        },
                        "required": ["label", "value"]
                    }
                },
                "title": {
                    "type": "string",
                    "description": "Chart title displayed at top"
                },
                "x_label": {
                    "type": "string",
                    "description": "Label for the X-axis (bar/line charts)"
                },
                "y_label": {
                    "type": "string",
                    "description": "Label for the Y-axis (bar/line charts)"
                },
                "width": {
                    "type": "integer",
                    "description": "Image width in pixels (default uses config setting)",
                    "minimum": 200,
                    "maximum": 2000
                },
                "height": {
                    "type": "integer",
                    "description": "Image height in pixels (default uses config setting)",
                    "minimum": 200,
                    "maximum": 2000
                }
            },
            "required": ["data"]
        })
    }

    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        let chart_type = args
            .get("chart_type")
            .and_then(|v| v.as_str())
            .unwrap_or("bar");

        let data: Vec<ChartDataPoint> = args
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ClawError::Validation("缺少必要参数: data".to_string()))?
            .iter()
            .map(|item| {
                let label = item
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let value = item.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                ChartDataPoint { label, value }
            })
            .collect();

        let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let x_label = args.get("x_label").and_then(|v| v.as_str()).unwrap_or("");
        let y_label = args.get("y_label").and_then(|v| v.as_str()).unwrap_or("");
        let width = args.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let height = args.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let images_dir = ensure_images_dir()?;

        let provider =
            image_gen::create_image_gen_provider(&ctx.config.image_gen, &ctx.http_client);

        let img = provider
            .generate_chart(
                chart_type,
                &data,
                title,
                x_label,
                y_label,
                width,
                height,
                &images_dir,
            )
            .await
            .map_err(|e| ClawError::Execution(format!("图表生成失败: {}", e)))?;

        Ok(serde_json::to_string(&serde_json::json!({
            "success": true,
            "image_path": img.path,
            "format": img.format,
            "width": img.width,
            "height": img.height,
            "alt_text": img.alt_text,
            "dashboard_url": format!("/api/images/{}", img.path),
        }))
        .map_err(|e| ClawError::Execution(format!("序列化失败: {}", e)))?)
    }
}

pub fn ensure_images_dir() -> Result<std::path::PathBuf, ClawError> {
    let claw_dir = crate::utils::claw_dir()
        .ok_or_else(|| ClawError::Execution("无法获取 claw 数据目录".to_string()))?;
    let images_dir = claw_dir.join("images");
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| ClawError::Execution(format!("无法创建图片目录: {}", e)))?;
    Ok(images_dir)
}
