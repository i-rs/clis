use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Table,
    Json,
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

pub fn output_list<T: Serialize + Clone>(items: &[T], count: usize, filter: Option<&str>, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => {
            let response = ListResponse {
                success: true,
                data: items.to_vec(),
                meta: ListMeta {
                    count,
                    filter: filter.map(String::from),
                },
            };
            serde_json::to_string_pretty(&response)
        }
        OutputFormat::Table => {
            serde_json::to_string(items)
        }
    }
    .unwrap_or_else(|_| r#"{"success":false,"error":{"code":"SERIALIZE_ERROR","message":"Failed to serialize"}}"#.to_string())
}

pub fn output_item<T: Serialize>(item: &T, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => {
            let response = ItemResponse {
                success: true,
                data: item,
            };
            serde_json::to_string_pretty(&response)
        }
        OutputFormat::Table => {
            serde_json::to_string(item)
        }
    }
    .unwrap_or_else(|_| r#"{"success":false,"error":{"code":"SERIALIZE_ERROR","message":"Failed to serialize"}}"#.to_string())
}

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
            serde_json::to_string_pretty(&response)
        }
        OutputFormat::Table => {
            Ok(format!("Error: {}", message))
        }
    }
    .unwrap_or_else(|_| r#"{"success":false,"error":{"code":"UNKNOWN","message":"Unknown error"}}"#.to_string())
}