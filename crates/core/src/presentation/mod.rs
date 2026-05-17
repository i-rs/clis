pub mod output;
pub mod theme;

pub use output::{OutputFormat, output_error, output_item, output_list};
pub use theme::{
    Theme, apply, get_theme, print_error, print_header, print_success, print_warning,
    println_dimmed, table_border_color, table_header_style, table_row_style,
};

use tabled::{
    Table, Tabled,
    settings::{
        object::Rows, object::Segment, style::BorderColor, style::Style, themes::Colorization,
    },
};

/// Render a table of items with consistent i-rs styling (cyan borders, bold header, green rows).
pub fn render_table<T: Tabled>(rows: &[T]) -> String {
    Table::new(rows)
        .with(Style::modern_rounded())
        .modify(Segment::all(), BorderColor::filled(table_border_color()))
        .with(Colorization::exact([table_header_style()], Rows::first()))
        .with(Colorization::exact([table_row_style()], Rows::new(1..)))
        .to_string()
}
