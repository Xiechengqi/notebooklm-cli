use std::fmt::{Display, Formatter};

use axum::http::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    AuthRequired,
    InvalidPassword,
    InvalidParams,
    CommandNotFound,
    BrowserNotFound,
    BrowserExecutionFailed,
    NotebooklmLoginRequired,
    NotebooklmRpcFailed,
    ConfigReadFailed,
    ConfigWriteFailed,
    InternalError,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::AuthRequired => "AUTH_REQUIRED",
            Self::InvalidPassword => "INVALID_PASSWORD",
            Self::InvalidParams => "INVALID_PARAMS",
            Self::CommandNotFound => "COMMAND_NOT_FOUND",
            Self::BrowserNotFound => "BROWSER_NOT_FOUND",
            Self::BrowserExecutionFailed => "BROWSER_EXECUTION_FAILED",
            Self::NotebooklmLoginRequired => "NOTEBOOKLM_LOGIN_REQUIRED",
            Self::NotebooklmRpcFailed => "NOTEBOOKLM_RPC_FAILED",
            Self::ConfigReadFailed => "CONFIG_READ_FAILED",
            Self::ConfigWriteFailed => "CONFIG_WRITE_FAILED",
            Self::InternalError => "INTERNAL_ERROR",
        };
        write!(f, "{value}")
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("authentication required")]
    AuthRequired,
    #[error("invalid password")]
    InvalidPassword,
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
    #[error("command not found: {0}")]
    CommandNotFound(String),
    #[error("agent-browser binary not found")]
    BrowserNotFound,
    #[error("agent-browser execution failed: {0}")]
    BrowserExecutionFailed(String),
    #[error("NotebookLM login required")]
    NotebooklmLoginRequired,
    #[error("NotebookLM RPC failed: {0}")]
    NotebooklmRpcFailed(String),
    #[error("failed to read config: {0}")]
    ConfigReadFailed(String),
    #[error("failed to write config: {0}")]
    ConfigWriteFailed(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::AuthRequired => ErrorCode::AuthRequired,
            Self::InvalidPassword => ErrorCode::InvalidPassword,
            Self::InvalidParams(_) => ErrorCode::InvalidParams,
            Self::CommandNotFound(_) => ErrorCode::CommandNotFound,
            Self::BrowserNotFound => ErrorCode::BrowserNotFound,
            Self::BrowserExecutionFailed(_) => ErrorCode::BrowserExecutionFailed,
            Self::NotebooklmLoginRequired => ErrorCode::NotebooklmLoginRequired,
            Self::NotebooklmRpcFailed(_) => ErrorCode::NotebooklmRpcFailed,
            Self::ConfigReadFailed(_) => ErrorCode::ConfigReadFailed,
            Self::ConfigWriteFailed(_) => ErrorCode::ConfigWriteFailed,
            Self::Internal(_) => ErrorCode::InternalError,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::AuthRequired | Self::InvalidPassword => StatusCode::UNAUTHORIZED,
            Self::InvalidParams(_) => StatusCode::BAD_REQUEST,
            Self::CommandNotFound(_) => StatusCode::NOT_FOUND,
            Self::BrowserNotFound => StatusCode::SERVICE_UNAVAILABLE,
            Self::BrowserExecutionFailed(_)
            | Self::NotebooklmRpcFailed(_)
            | Self::NotebooklmLoginRequired
            | Self::ConfigReadFailed(_)
            | Self::ConfigWriteFailed(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidParams(_) => 2,
            Self::AuthRequired | Self::InvalidPassword => 3,
            Self::CommandNotFound(_) => 4,
            _ => 1,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
