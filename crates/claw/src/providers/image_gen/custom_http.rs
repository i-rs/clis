use super::{generate_filename, GeneratedImage, ImageGenProvider, ImageGenProviderKind};
use crate::config::ImageGenConfig;
use serde_json::json;

pub struct CustomHttpProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    width: u32,
    height: u32,
}

impl CustomHttpProvider {
    pub fn new(client: reqwest::Client, config: &ImageGenConfig) -> Self {
        Self {
            client,
            api_key: config.api_key.clone(),
            base_url: config.base_url.clone(),
            width: config.default_width,
            height: config.default_height,
        }
    }
}

#[async_trait::async_trait]
impl ImageGenProvider for CustomHttpProvider {
    fn kind(&self) -> ImageGenProviderKind {
        ImageGenProviderKind::CustomHttp
    }

    async fn generate_chart(
        &self,
        _chart_type: &str,
        _data: &[super::ChartDataPoint],
        _title: &str,
        _x_label: &str,
        _y_label: &str,
        _width: u32,
        _height: u32,
        _images_dir: &std::path::Path,
    ) -> anyhow::Result<GeneratedImage> {
        anyhow::bail!(
            "Custom HTTP provider does not support structured chart generation. \
             Use generate_image for creative image generation from text prompts."
        )
    }

    async fn generate_image(
        &self,
        prompt: &str,
        width: u32,
        height: u32,
        images_dir: &std::path::Path,
    ) -> anyhow::Result<GeneratedImage> {
        if self.base_url.is_empty() {
            anyhow::bail!("custom_http provider requires base_url to be configured in [image_gen]");
        }

        let w = if width > 0 { width } else { self.width };
        let h = if height > 0 { height } else { self.height };

        let body = json!({
            "prompt": prompt,
            "width": w,
            "height": h,
        });

        let mut req = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json");

        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = req.json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Custom HTTP image gen returned HTTP {}: {}", status, body);
        }

        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/png")
            .to_string();

        let ext = if content_type.contains("svg") {
            "svg"
        } else if content_type.contains("jpeg") || content_type.contains("jpg") {
            "jpg"
        } else {
            "png"
        };

        let bytes = resp.bytes().await?;
        let filepath = generate_filename("gen", ext, images_dir);
        std::fs::write(&filepath, &bytes)?;

        Ok(GeneratedImage {
            path: filepath.file_name().unwrap_or_default().to_string_lossy().to_string(),
            format: ext.to_string(),
            width: w,
            height: h,
            alt_text: prompt.chars().take(100).collect(),
        })
    }
}
