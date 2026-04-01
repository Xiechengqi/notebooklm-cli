use serde::Serialize;

use crate::errors::ErrorCode;

#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    pub site: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorBody>,
    pub meta: ResponseMeta,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T, command: Option<String>) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
            meta: ResponseMeta {
                site: "notebooklm",
                command,
                request_id: None,
            },
        }
    }
}
