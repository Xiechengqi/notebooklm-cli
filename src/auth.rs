use axum::Json;
use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::config::AppConfig;
use crate::errors::AppError;

pub const AUTH_COOKIE_NAME: &str = "notebooklm_cli_token";

#[derive(Clone)]
pub struct AuthState {
    pub password: String,
    pub password_initialized: bool,
}

impl AuthState {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            password: config.auth.password.clone(),
            password_initialized: config.is_password_initialized(),
        }
    }
}

pub fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(AUTHORIZATION)?.to_str().ok()?;
    raw.strip_prefix("Bearer ").map(ToString::to_string)
}

pub fn extract_cookie_token(headers: &HeaderMap) -> Option<String> {
    let cookies = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for item in cookies.split(';') {
        let trimmed = item.trim();
        if let Some(value) = trimmed.strip_prefix(&format!("{AUTH_COOKIE_NAME}=")) {
            return Some(value.to_string());
        }
    }
    None
}

pub fn is_authenticated(headers: &HeaderMap, state: &AuthState) -> bool {
    if !state.password_initialized {
        return true;
    }

    extract_bearer(headers)
        .into_iter()
        .chain(extract_cookie_token(headers))
        .any(|token| token == state.password)
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "ok": false,
            "error": {
                "code": self.code(),
                "message": self.to_string(),
            },
            "meta": {
                "site": "notebooklm"
            }
        }));
        (self.status_code(), body).into_response()
    }
}
