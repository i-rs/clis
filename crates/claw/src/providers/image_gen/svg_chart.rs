use super::{GeneratedImage, ImageGenProvider, ImageGenProviderKind, generate_filename};
use crate::config::ImageGenConfig;

pub struct SvgChartProvider {
    chart_type: String,
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
}

impl SvgChartProvider {
    pub fn new(config: &ImageGenConfig) -> Self {
        Self {
            chart_type: config.default_chart_type.clone(),
            width: config.default_width,
            height: config.default_height,
        }
    }
}

#[async_trait::async_trait]
impl ImageGenProvider for SvgChartProvider {
    fn kind(&self) -> ImageGenProviderKind {
        ImageGenProviderKind::SvgChart
    }

    async fn generate_chart(
        &self,
        chart_type: &str,
        data: &[super::ChartDataPoint],
        title: &str,
        x_label: &str,
        y_label: &str,
        width: u32,
        height: u32,
        images_dir: &std::path::Path,
    ) -> anyhow::Result<GeneratedImage> {
        let ct = if chart_type.is_empty() {
            &self.chart_type
        } else {
            chart_type
        };

        let svg = match ct {
            "line" => render_line_chart(data, title, x_label, y_label, width, height),
            "pie" => render_pie_chart(data, title, width, height),
            "radar" => render_radar_chart(data, title, width, height),
            _ => render_bar_chart(data, title, x_label, y_label, width, height),
        };

        let filepath = generate_filename("chart", "svg", images_dir);
        std::fs::write(&filepath, &svg)?;

        let alt = if title.is_empty() {
            format!("Chart with {} data points", data.len())
        } else {
            title.to_string()
        };

        Ok(GeneratedImage {
            path: filepath
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            format: "svg".to_string(),
            width,
            height,
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
            "SVG Chart provider does not support free-form image generation. \
             Use a different image_gen.provider (e.g., custom_http) for creative image generation."
        )
    }
}

const COLORS: &[&str] = &[
    "#22d3ee", "#34d399", "#fbbf24", "#f87171", "#a78bfa", "#60a5fa", "#fb923c", "#e879f9",
    "#2dd4bf", "#facc15",
];

const SVG_BG: &str = "#0f0f17";
const SVG_GRID: &str = "#27273a";
const SVG_TEXT: &str = "#a1a1aa";
const SVG_TITLE: &str = "#fafafa";
const SVG_AXIS: &str = "#52525b";

fn render_bar_chart(
    data: &[super::ChartDataPoint],
    title: &str,
    x_label: &str,
    y_label: &str,
    width: u32,
    height: u32,
) -> String {
    let margin = Margin {
        top: 60,
        right: 40,
        bottom: 70,
        left: 70,
    };
    let pw = width.saturating_sub(margin.left + margin.right) as f64;
    let ph = height.saturating_sub(margin.top + margin.bottom) as f64;

    let max_val = data.iter().map(|d| d.value).fold(0.0f64, f64::max);
    let y_max = nice_max(max_val);

    let n = data.len().max(1);
    let bar_width = (pw / n as f64 * 0.6).max(4.0);
    let gap = pw / n as f64;

    let chart_left = margin.left as f64;
    let chart_top = margin.top as f64;
    let chart_bottom = chart_top + ph;

    let mut svg = svg_header(width, height);
    svg.push_str(&svg_rect(0.0, 0.0, width as f64, height as f64, SVG_BG));

    if !title.is_empty() {
        svg.push_str(&svg_text(
            (width / 2) as f64,
            35.0,
            SVG_TITLE,
            18,
            "middle",
            title,
        ));
    }

    for i in 0..=5 {
        let y = chart_top + ph * (i as f64) / 5.0;
        let val = y_max * (5.0 - i as f64) / 5.0;
        svg.push_str(&svg_line(chart_left, y, chart_left + pw, y, SVG_GRID, 1));
        let label = if val.fract() == 0.0 {
            format!("{:.0}", val)
        } else {
            format!("{:.1}", val)
        };
        svg.push_str(&svg_text(
            chart_left - 8.0,
            y + 5.0,
            SVG_TEXT,
            12,
            "end",
            &label,
        ));
    }

    svg.push_str(&svg_line(
        chart_left,
        chart_top,
        chart_left,
        chart_bottom,
        SVG_AXIS,
        2,
    ));
    svg.push_str(&svg_line(
        chart_left,
        chart_bottom,
        chart_left + pw,
        chart_bottom,
        SVG_AXIS,
        2,
    ));

    if !y_label.is_empty() {
        svg.push_str(&svg_text(
            16.0,
            chart_top + ph / 2.0,
            SVG_TEXT,
            13,
            "middle",
            y_label,
        ));
    }
    if !x_label.is_empty() {
        svg.push_str(&svg_text(
            chart_left + pw / 2.0,
            (height - 10) as f64,
            SVG_TEXT,
            13,
            "middle",
            x_label,
        ));
    }

    for (i, dp) in data.iter().enumerate() {
        let x = chart_left + gap * (i as f64 + 0.5) - bar_width / 2.0;
        let bar_h = if y_max > 0.0 {
            (dp.value / y_max) * ph
        } else {
            0.0
        };
        let y = chart_bottom - bar_h;
        let color = COLORS[i % COLORS.len()];

        svg.push_str(&svg_rect(x, y, bar_width, bar_h, color));
        svg.push_str(&svg_text(
            chart_left + gap * (i as f64 + 0.5),
            chart_bottom + 18.0,
            SVG_TEXT,
            11,
            "middle",
            &trunc_label(&dp.label, 12),
        ));

        let val_str = if dp.value.fract() == 0.0 {
            format!("{:.0}", dp.value)
        } else {
            format!("{:.1}", dp.value)
        };
        svg.push_str(&svg_text(
            chart_left + gap * (i as f64 + 0.5),
            y - 6.0,
            SVG_TEXT,
            11,
            "middle",
            &val_str,
        ));
    }

    svg.push_str("</svg>");
    svg
}

fn render_line_chart(
    data: &[super::ChartDataPoint],
    title: &str,
    x_label: &str,
    y_label: &str,
    width: u32,
    height: u32,
) -> String {
    let margin = Margin {
        top: 60,
        right: 40,
        bottom: 70,
        left: 70,
    };
    let pw = width.saturating_sub(margin.left + margin.right) as f64;
    let ph = height.saturating_sub(margin.top + margin.bottom) as f64;

    let max_val = data.iter().map(|d| d.value).fold(0.0f64, f64::max);
    let y_max = nice_max(max_val);

    let chart_left = margin.left as f64;
    let chart_top = margin.top as f64;
    let chart_bottom = chart_top + ph;

    let mut svg = svg_header(width, height);
    svg.push_str(&svg_rect(0.0, 0.0, width as f64, height as f64, SVG_BG));

    if !title.is_empty() {
        svg.push_str(&svg_text(
            (width / 2) as f64,
            35.0,
            SVG_TITLE,
            18,
            "middle",
            title,
        ));
    }

    for i in 0..=5 {
        let y = chart_top + ph * (i as f64) / 5.0;
        let val = y_max * (5.0 - i as f64) / 5.0;
        svg.push_str(&svg_line(chart_left, y, chart_left + pw, y, SVG_GRID, 1));
        let label = if val.fract() == 0.0 {
            format!("{:.0}", val)
        } else {
            format!("{:.1}", val)
        };
        svg.push_str(&svg_text(
            chart_left - 8.0,
            y + 5.0,
            SVG_TEXT,
            12,
            "end",
            &label,
        ));
    }

    svg.push_str(&svg_line(
        chart_left,
        chart_top,
        chart_left,
        chart_bottom,
        SVG_AXIS,
        2,
    ));
    svg.push_str(&svg_line(
        chart_left,
        chart_bottom,
        chart_left + pw,
        chart_bottom,
        SVG_AXIS,
        2,
    ));

    if !y_label.is_empty() {
        svg.push_str(&svg_text(
            16.0,
            chart_top + ph / 2.0,
            SVG_TEXT,
            13,
            "middle",
            y_label,
        ));
    }
    if !x_label.is_empty() {
        svg.push_str(&svg_text(
            chart_left + pw / 2.0,
            (height - 10) as f64,
            SVG_TEXT,
            13,
            "middle",
            x_label,
        ));
    }

    let n = data.len().max(1);
    let gap = if n > 1 {
        pw / (n as f64 - 1.0)
    } else {
        pw / 2.0
    };

    let mut points = Vec::new();
    let mut poly_points = String::new();

    for (i, dp) in data.iter().enumerate() {
        let x = if n > 1 {
            chart_left + gap * i as f64
        } else {
            chart_left + pw / 2.0
        };
        let y = chart_bottom
            - if y_max > 0.0 {
                (dp.value / y_max) * ph
            } else {
                0.0
            };
        points.push((x, y, &dp.label, dp.value));

        if i == 0 {
            poly_points.push_str(&format!("{:.1},{:.1}", x, y));
        } else {
            poly_points.push_str(&format!(" {:.1},{:.1}", x, y));
        }
    }

    if !poly_points.is_empty() {
        let fill_area = format!(
            "{} {:.1},{:.1} {:.1},{:.1}",
            poly_points,
            chart_left + pw,
            chart_bottom,
            chart_left,
            chart_bottom
        );
        svg.push_str(&format!(
            "<polygon points=\"{}\" fill=\"url(#lineGrad)\" opacity=\"0.2\"/>",
            fill_area
        ));

        let color = COLORS[0];
        svg.push_str(&format!(
            "<linearGradient id=\"lineGrad\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"0%\" stop-color=\"{}\" stop-opacity=\"0.4\"/><stop offset=\"100%\" stop-color=\"{}\" stop-opacity=\"0\"/></linearGradient>",
            color, color
        ));
        svg.push_str(&format!(
            "<polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
            poly_points, color
        ));
    }

    for (x, y, label, val) in &points {
        let color = COLORS[0];
        svg.push_str(&format!(
            "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"{}\" stroke=\"{}\" stroke-width=\"2\"/>",
            x, y, SVG_BG, color
        ));
        svg.push_str(&svg_text(
            *x,
            chart_bottom + 18.0,
            SVG_TEXT,
            11,
            "middle",
            &trunc_label(label, 12),
        ));
        let val_str = if val.fract() == 0.0 {
            format!("{:.0}", val)
        } else {
            format!("{:.1}", val)
        };
        svg.push_str(&svg_text(*x, *y - 12.0, SVG_TEXT, 11, "middle", &val_str));
    }

    svg.push_str("</svg>");
    svg
}

fn render_pie_chart(
    data: &[super::ChartDataPoint],
    title: &str,
    width: u32,
    height: u32,
) -> String {
    let cx = (width / 2) as f64;
    let cy = (height / 2) as f64 + 20.0;
    let radius = (width.min(height) as f64 / 2.0 * 0.65).max(50.0);

    let total: f64 = data.iter().map(|d| d.value).sum();
    if total <= 0.0 {
        return simple_svg(width, height, title, "No data");
    }

    let mut svg = svg_header(width, height);
    svg.push_str(&svg_rect(0.0, 0.0, width as f64, height as f64, SVG_BG));

    if !title.is_empty() {
        svg.push_str(&svg_text(cx, 35.0, SVG_TITLE, 18, "middle", title));
    }

    let mut start_angle = -90.0f64;
    let legend_x = cx + radius + 40.0;
    let mut legend_y = cy - (data.len() as f64 * 22.0) / 2.0;

    for (i, dp) in data.iter().enumerate() {
        let pct = dp.value / total;
        let angle = 360.0 * pct;
        let color = COLORS[i % COLORS.len()];

        let (path, mid_angle) = make_slice(cx, cy, radius, start_angle, angle);

        svg.push_str(&format!(
            "<path d=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>",
            path, color, SVG_BG
        ));

        if pct >= 0.05 {
            let mid_rad = mid_angle.to_radians();
            let label_r = radius * 0.65;
            let lx = cx + mid_rad.cos() * label_r;
            let ly = cy + mid_rad.sin() * label_r;
            let label = format!("{:.0}%", pct * 100.0);
            svg.push_str(&svg_text(lx, ly + 5.0, SVG_TITLE, 12, "middle", &label));
        }

        svg.push_str(&svg_rect(legend_x - 6.0, legend_y - 5.0, 12.0, 12.0, color));
        svg.push_str(&svg_text(
            legend_x + 12.0,
            legend_y + 5.0,
            SVG_TEXT,
            12,
            "start",
            &format!("{} ({:.1})", trunc_label(&dp.label, 15), dp.value),
        ));

        legend_y += 22.0;
        start_angle += angle;
    }

    svg.push_str("</svg>");
    svg
}

fn render_radar_chart(
    data: &[super::ChartDataPoint],
    title: &str,
    width: u32,
    height: u32,
) -> String {
    let cx = (width / 2) as f64;
    let cy = (height / 2) as f64 + 20.0;
    let radius = (width.min(height) as f64 / 2.0 * 0.5).max(50.0);

    let n = data.len().max(3);
    let max_val = data.iter().map(|d| d.value).fold(0.0f64, f64::max);
    let r_max = nice_max(max_val);

    let mut svg = svg_header(width, height);
    svg.push_str(&svg_rect(0.0, 0.0, width as f64, height as f64, SVG_BG));

    if !title.is_empty() {
        svg.push_str(&svg_text(cx, 35.0, SVG_TITLE, 18, "middle", title));
    }

    let angle_step = 2.0 * std::f64::consts::PI / n as f64;
    let start_angle = -std::f64::consts::PI / 2.0;

    for level in 0..=4 {
        let r = radius * (level as f64) / 4.0;
        let points: Vec<String> = (0..n)
            .map(|i| {
                let a = start_angle + angle_step * i as f64;
                let x = cx + a.cos() * r;
                let y = cy + a.sin() * r;
                format!("{:.1},{:.1}", x, y)
            })
            .collect();

        let poly = points.join(" ");
        svg.push_str(&format!(
            "<polygon points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"0.5\" opacity=\"0.5\"/>",
            poly, SVG_GRID
        ));

        let val = r_max * (4.0 - level as f64) / 4.0;
        let label = if val.fract() == 0.0 {
            format!("{:.0}", val)
        } else {
            format!("{:.1}", val)
        };
        svg.push_str(&svg_text(
            cx + 4.0,
            cy - radius + radius * (level as f64) / 4.0 + 5.0,
            SVG_TEXT,
            10,
            "start",
            &label,
        ));
    }

    for i in 0..n {
        let a = start_angle + angle_step * i as f64;
        let ex = cx + a.cos() * radius;
        let ey = cy + a.sin() * radius;
        svg.push_str(&format!(
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"0.5\"/>",
            cx, cy, ex, ey, SVG_GRID
        ));
    }

    let data_points: Vec<String> = data
        .iter()
        .enumerate()
        .map(|(i, dp)| {
            let a = start_angle + angle_step * i as f64;
            let r = if r_max > 0.0 {
                (dp.value / r_max) * radius
            } else {
                0.0
            };
            let x = cx + a.cos() * r;
            let y = cy + a.sin() * r;
            format!("{:.1},{:.1}", x, y)
        })
        .collect();

    let poly = data_points.join(" ");
    let color = COLORS[0];
    svg.push_str(&format!(
        "<polygon points=\"{}\" fill=\"{}\" fill-opacity=\"0.25\" stroke=\"{}\" stroke-width=\"2\"/>",
        poly, color, color
    ));

    for (i, dp) in data.iter().enumerate() {
        let a = start_angle + angle_step * i as f64;
        let r = if r_max > 0.0 {
            (dp.value / r_max) * radius
        } else {
            0.0
        };
        let x = cx + a.cos() * r;
        let y = cy + a.sin() * r;
        svg.push_str(&format!(
            "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"4\" fill=\"{}\" stroke=\"{}\" stroke-width=\"2\"/>",
            x, y, SVG_BG, color
        ));

        let label_x = cx + a.cos() * (radius + 25.0);
        let label_y = cy + a.sin() * (radius + 25.0);
        let anchor = if a.cos() > 0.1 {
            "start"
        } else if a.cos() < -0.1 {
            "end"
        } else {
            "middle"
        };
        svg.push_str(&svg_text(
            label_x,
            label_y + 5.0,
            SVG_TEXT,
            11,
            anchor,
            &dp.label,
        ));

        let val_str = if dp.value.fract() == 0.0 {
            format!("{:.0}", dp.value)
        } else {
            format!("{:.1}", dp.value)
        };
        svg.push_str(&svg_text(x, y - 10.0, SVG_TITLE, 10, "middle", &val_str));
    }

    svg.push_str("</svg>");
    svg
}

struct Margin {
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
}

fn nice_max(max_val: f64) -> f64 {
    if max_val <= 0.0 {
        return 10.0;
    }
    let magnitude = 10.0f64.powf(max_val.log10().floor());
    let normalized = max_val / magnitude;
    let nice = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 2.5 {
        2.5
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

fn svg_header(w: u32, h: u32) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
        w, h, w, h
    )
}

fn svg_rect(x: f64, y: f64, w: impl Into<f64>, h: impl Into<f64>, fill: &str) -> String {
    format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\"/>",
        x,
        y,
        w.into(),
        h.into(),
        fill
    )
}

fn svg_text(x: f64, y: f64, fill: &str, size: u32, anchor: &str, text: &str) -> String {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\" font-size=\"{}\" font-family=\"sans-serif\" text-anchor=\"{}\">{}</text>",
        x, y, fill, size, anchor, escaped
    )
}

fn svg_line(x1: f64, y1: f64, x2: f64, y2: f64, stroke: &str, width: u32) -> String {
    format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"{}\"/>",
        x1, y1, x2, y2, stroke, width
    )
}

fn make_slice(cx: f64, cy: f64, r: f64, start: f64, sweep: f64) -> (String, f64) {
    let start_rad = start.to_radians();
    let end_rad = (start + sweep).to_radians();

    let x1 = cx + start_rad.cos() * r;
    let y1 = cy + start_rad.sin() * r;
    let x2 = cx + end_rad.cos() * r;
    let y2 = cy + end_rad.sin() * r;

    let large_arc = if sweep > 180.0 { 1 } else { 0 };
    let d = format!(
        "M {:.1},{:.1} L {:.1},{:.1} A {:.1},{:.1} 0 {},1 {:.1},{:.1} Z",
        cx, cy, x1, y1, r, r, large_arc, x2, y2
    );
    (d, start + sweep / 2.0)
}

fn trunc_label(label: &str, max_len: usize) -> String {
    if label.chars().count() > max_len {
        format!(
            "{}…",
            label
                .chars()
                .take(max_len.saturating_sub(1))
                .collect::<String>()
        )
    } else {
        label.to_string()
    }
}

fn simple_svg(w: u32, h: u32, title: &str, msg: &str) -> String {
    let mut svg = svg_header(w, h);
    svg.push_str(&svg_rect(0.0, 0.0, w as f64, h as f64, SVG_BG));
    if !title.is_empty() {
        svg.push_str(&svg_text(
            (w / 2) as f64,
            (h / 2) as f64 - 10.0,
            SVG_TITLE,
            16,
            "middle",
            title,
        ));
    }
    svg.push_str(&svg_text(
        (w / 2) as f64,
        (h / 2) as f64 + 14.0,
        SVG_TEXT,
        14,
        "middle",
        msg,
    ));
    svg.push_str("</svg>");
    svg
}
