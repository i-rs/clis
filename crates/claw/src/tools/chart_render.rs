/// A labeled numeric data point for chart rendering.
pub struct DataPoint {
    pub label: String,
    pub value: f64,
}

/// Check if a value is within the threshold range spanned by two points.
fn is_between(a: f64, b: f64, threshold: f64, epsilon: f64) -> bool {
    let min_v = a.min(b);
    let max_v = a.max(b);
    threshold >= min_v - epsilon && threshold <= max_v + epsilon
}

/// Generate an ASCII bar chart from data points.
pub fn generate_bar_chart(data: &[DataPoint], width: usize, height: usize) -> String {
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
        "┌ 柱状图 (max={:.1})\n",
        max_val
    ));

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
        if row == 0 {
            output.push_str(&format!("│ {:.0}", max_val));
        } else if row == chart_height - 1 {
            output.push_str("│ 0");
        }
        output.push('\n');
    }

    output.push('└');
    for _ in 0..top_w {
        output.push('─');
    }
    output.push_str("┘\n");

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

/// Generate an ASCII line chart from data points.
pub fn generate_line_chart(data: &[DataPoint], width: usize, height: usize) -> String {
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

    let grid_cols = data.len();
    let col_width = ((chart_width.saturating_sub(6)) / grid_cols.max(1)).max(1);

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
                && (point.value - min_val) <= (range * (chart_height - row) as f64 / chart_height as f64 + 1e-10);

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
        if row == 0 {
            output.push_str(&format!("│ {:.0}", max_val));
        } else if row == chart_height - 1 {
            output.push_str(&format!("│ {:.0}", min_val));
        }
        output.push('\n');
    }

    output.push('└');
    for _ in 0..top_w {
        output.push('─');
    }
    output.push_str("┘\n");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_between_direct() {
        assert!(is_between(0.0, 10.0, 5.0, 1.0));
    }

    #[test]
    fn test_is_between_at_edge() {
        assert!(is_between(5.0, 10.0, 4.5, 1.0));
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

    #[test]
    fn test_bar_chart_empty() {
        assert_eq!(generate_bar_chart(&[], 40, 10), "(无数据)");
    }

    #[test]
    fn test_bar_chart_single_point() {
        let data = vec![DataPoint { label: "Test".into(), value: 100.0 }];
        let result = generate_bar_chart(&data, 30, 5);
        assert!(result.contains("Test"));
        assert!(result.contains("█"));
    }

    #[test]
    fn test_bar_chart_multiple_points() {
        let data = vec![
            DataPoint { label: "A".into(), value: 50.0 },
            DataPoint { label: "B".into(), value: 100.0 },
            DataPoint { label: "C".into(), value: 30.0 },
        ];
        let result = generate_bar_chart(&data, 40, 10);
        assert!(result.contains("A"));
        assert!(result.contains("B"));
        assert!(result.contains("C"));
        assert!(result.contains("100"));
    }

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
        assert!(result.contains("Mon"));
        assert!(result.contains("折线图"));
        assert!(result.contains("20"));
        assert!(result.contains("0"));
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
        assert!(result.lines().count() > 5);
    }
}
