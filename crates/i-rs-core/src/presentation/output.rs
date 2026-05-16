use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Default,
    Table,
    Json,
}

impl OutputFormat {
    pub fn is_json(&self) -> bool {
        matches!(self, OutputFormat::Json)
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "table" => Ok(Self::Table),
            _ => Ok(Self::Default),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub meta: ListMeta,
}

#[derive(Debug, Serialize)]
pub struct ListMeta {
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ItemResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

pub fn output_list<T: Serialize + Clone>(
    items: &[T],
    count: usize,
    filter: Option<&str>,
    _format: OutputFormat,
) -> String {
    let response = ListResponse {
        success: true,
        data: items.to_vec(),
        meta: ListMeta {
            count,
            filter: filter.map(String::from),
        },
    };
    serde_json::to_string_pretty(&response)
        .unwrap_or_else(|_| r#"{"success":false,"error":{"code":"SERIALIZE_ERROR","message":"Failed to serialize"}}"#.to_string())
}

pub fn output_item<T: Serialize>(item: &T, _format: OutputFormat) -> String {
    let response = ItemResponse {
        success: true,
        data: item,
    };
    serde_json::to_string_pretty(&response)
        .unwrap_or_else(|_| r#"{"success":false,"error":{"code":"SERIALIZE_ERROR","message":"Failed to serialize"}}"#.to_string())
}

#[must_use]
pub fn output_error(message: &str, code: &str, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => {
            let response = ErrorResponse {
                success: false,
                error: ErrorDetail {
                    code: code.to_string(),
                    message: message.to_string(),
                },
            };
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| {
                r#"{"success":false,"error":{"code":"UNKNOWN","message":"Unknown error"}}"#
                    .to_string()
            })
        }
        OutputFormat::Table | OutputFormat::Default => {
            format!("Error: {message}")
        }
    }
}
