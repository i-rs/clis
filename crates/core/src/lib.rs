pub mod presentation;
pub mod storage;
pub mod tool;
pub mod utils;

pub mod r#macro;

pub use presentation::output::{output_error, output_item, output_list};
pub use presentation::{
    OutputFormat, Theme, apply, get_theme, print_error, print_header, print_success, print_warning,
    println_dimmed, render_table, table_border_color, table_header_style, table_row_style,
};
pub use storage::{FileBackend, HasTags, Storage, StorageBackend, filter_by_tag, set_backend};
pub use tool::{IrsTool, Pagination, ToolCapability, paginate_entries};
pub use utils::date::{
    format_date, format_date_custom, format_datetime, now_utc, parse_date, parse_datetime,
};
pub use utils::validation::{
    ValidationError, validate_amount, validate_name, validate_url, validate_weight,
};
