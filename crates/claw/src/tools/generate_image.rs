use crate::error::ClawError;
use crate::providers::image_gen::{self};
use serde_json::Value;

use super::{ClawTool, ToolContext};

pub struct GenerateImageTool;

#[async_trait::async_trait]
impl ClawTool for GenerateImageTool {
    fn name(&self) -> &str {
        "generate_image"
    }

    fn description(&self) -> &str {
        "Generate a creative image from a text description using the configured image \
         generation provider. This requires a provider that supports free-form image \
         generation (custom_http). For data charts, use the chart_image tool instead."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text description of the image to generate"
                },
                "width": {
                    "type": "integer",
                    "description": "Image width in pixels (default: config setting)",
                    "minimum": 256,
                    "maximum": 2048
                },
                "height": {
                    "type": "integer",
                    "description": "Image height in pixels (default: config setting)",
                    "minimum": 256,
                    "maximum": 2048
                }
            },
            "required": ["prompt"]
        })
    }

    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClawError::Validation("缺少必要参数: prompt".to_string()))?;

        let width = args.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let height = args.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let images_dir = super::chart_image::ensure_images_dir()?;

        let provider = image_gen::create_image_gen_provider(
            &ctx.config.image_gen,
            &ctx.http_client,
        );

        let img = provider
            .generate_image(prompt, width, height, &images_dir)
            .await
            .map_err(|e| ClawError::Execution(format!("图片生成失败: {}", e)))?;

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
