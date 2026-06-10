pub mod custom_http;
pub mod quick_chart;
pub mod svg_chart;

pub use custom_http::CustomHttpProvider;
pub use quick_chart::QuickChartProvider;
pub use svg_chart::SvgChartProvider;

use crate::config::ImageGenConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageGenProviderKind {
    SvgChart,
    QuickChart,
    CustomHttp,
}

impl ImageGenProviderKind {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageGenProviderKind::SvgChart => "svg_chart",
            ImageGenProviderKind::QuickChart => "quick_chart",
            ImageGenProviderKind::CustomHttp => "custom_http",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "quick_chart" | "quickchart" => ImageGenProviderKind::QuickChart,
            "custom_http" | "custom" => ImageGenProviderKind::CustomHttp,
            _ => ImageGenProviderKind::SvgChart,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    pub label: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedImage {
    pub path: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub alt_text: String,
}

#[async_trait::async_trait]
#[allow(clippy::too_many_arguments)]
pub trait ImageGenProvider: Send + Sync {
    #[allow(dead_code)]
    fn kind(&self) -> ImageGenProviderKind;

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
    ) -> anyhow::Result<GeneratedImage>;

    async fn generate_image(
        &self,
        prompt: &str,
        width: u32,
        height: u32,
        images_dir: &std::path::Path,
    ) -> anyhow::Result<GeneratedImage>;
}

pub fn create_image_gen_provider(
    config: &ImageGenConfig,
    client: &reqwest::Client,
) -> Box<dyn ImageGenProvider> {
    match ImageGenProviderKind::from_str(&config.provider) {
        ImageGenProviderKind::SvgChart => Box::new(SvgChartProvider::new(config)),
        ImageGenProviderKind::QuickChart => {
            Box::new(QuickChartProvider::new(client.clone(), config))
        }
        ImageGenProviderKind::CustomHttp => {
            Box::new(CustomHttpProvider::new(client.clone(), config))
        }
    }
}

#[allow(dead_code)]
pub fn default_chart_width() -> u32 {
    800
}

#[allow(dead_code)]
pub fn default_chart_height() -> u32 {
    500
}

fn generate_filename(
    prefix: &str,
    format: &str,
    images_dir: &std::path::Path,
) -> std::path::PathBuf {
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let rand: u16 = fastrand::u16(0..10000);
    let name = format!("{}_{}_{:04}.{}", prefix, ts, rand, format);
    images_dir.join(name)
}
