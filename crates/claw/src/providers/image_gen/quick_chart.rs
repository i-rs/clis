use super::{generate_filename, ChartDataPoint, GeneratedImage, ImageGenProvider, ImageGenProviderKind};
use crate::config::ImageGenConfig;
use serde_json::json;

pub struct QuickChartProvider {
    client: reqwest::Client,
    width: u32,
    height: u32,
}

impl QuickChartProvider {
    pub fn new(client: reqwest::Client, config: &ImageGenConfig) -> Self {
        Self {
            client,
            width: config.default_width,
            height: config.default_height,
        }
    }
}

#[async_trait::async_trait]
impl ImageGenProvider for QuickChartProvider {
    fn kind(&self) -> ImageGenProviderKind {
        ImageGenProviderKind::QuickChart
    }

    async fn generate_chart(
        &self,
        chart_type: &str,
        data: &[ChartDataPoint],
        title: &str,
        x_label: &str,
        y_label: &str,
        width: u32,
        height: u32,
        images_dir: &std::path::Path,
    ) -> anyhow::Result<GeneratedImage> {
        let ct = if chart_type.is_empty() { "bar" } else { chart_type };
        let labels: Vec<String> = data.iter().map(|d| d.label.clone()).collect();
        let values: Vec<f64> = data.iter().map(|d| d.value).collect();

        let chart_json = json!({
            "type": ct,
            "data": {
                "labels": labels,
                "datasets": [{
                    "label": y_label,
                    "data": values,
                    "backgroundColor": [
                        "#22d3ee", "#34d399", "#fbbf24", "#f87171", "#a78bfa",
                        "#60a5fa", "#fb923c", "#e879f9", "#2dd4bf", "#facc15"
                    ],
                    "borderColor": "#22d3ee",
                }]
            },
            "options": {
                "plugins": {
                    "title": {
                        "display": !title.is_empty(),
                        "text": title,
                        "color": "#fafafa"
                    },
                    "legend": {
                        "labels": { "color": "#a1a1aa" }
                    }
                },
                "scales": {
                    "x": {
                        "title": {
                            "display": !x_label.is_empty(),
                            "text": x_label,
                            "color": "#a1a1aa"
                        },
                        "ticks": { "color": "#a1a1aa" },
                        "grid": { "color": "#27273a" }
                    },
                    "y": {
                        "title": {
                            "display": !y_label.is_empty(),
                            "text": y_label,
                            "color": "#a1a1aa"
                        },
                        "ticks": { "color": "#a1a1aa" },
                        "grid": { "color": "#27273a" }
                    }
                },
                "layout": {
                    "backgroundColor": "#0f0f17"
                }
            }
        });

        let w = if width > 0 { width } else { self.width };
        let h = if height > 0 { height } else { self.height };

        let url = format!(
            "https://quickchart.io/chart?w={}&h={}&chart={}",
            w,
            h,
            urlencoding(&serde_json::to_string(&chart_json)?)
        );

        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("QuickChart returned HTTP {}", resp.status());
        }
        let bytes = resp.bytes().await?;

        let filepath = generate_filename("chart", "png", images_dir);
        std::fs::write(&filepath, &bytes)?;

        let alt = if title.is_empty() {
            format!("Chart with {} data points", data.len())
        } else {
            title.to_string()
        };

        Ok(GeneratedImage {
            path: filepath.file_name().unwrap_or_default().to_string_lossy().to_string(),
            format: "png".to_string(),
            width: w,
            height: h,
            alt_text: alt,
        })
    }

    async fn generate_image(
        &self,
        _prompt: &str,
        _width: u32,
        _height: u32,
        _images_dir: &std::path::Path,
    ) -> anyhow::Result<GeneratedImage> {
        anyhow::bail!(
            "QuickChart provider does not support free-form image generation. \
             Use a different image_gen.provider for creative image generation."
        )
    }
}

fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(b as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", b));
            }
        }
    }
    result
}
