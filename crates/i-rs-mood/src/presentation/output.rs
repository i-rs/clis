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
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| r#"{"success":false,"error":{"code":"SERIALIZE_ERROR","message":"Failed to serialize"}}"#.to_string())
        }
        OutputFormat::Table => {
            serde_json::to_string(items).unwrap_or_default()
        }
    }
}