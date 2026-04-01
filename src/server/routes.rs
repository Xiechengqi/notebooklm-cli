use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::{AUTH_COOKIE_NAME, is_authenticated};
use crate::config;
use crate::discovery;
use crate::errors::AppError;
use crate::response::ApiResponse;
use crate::server::{AppState, ExecutionRecord};

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/logout", post(logout))
        .route("/health", get(health))
        .route("/api/bootstrap", get(bootstrap))
        .route("/api/setup/password", post(setup_password))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout_api))
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/commands", get(get_commands))
        .route("/api/history", get(get_history))
        .route("/api/mcp/tools", get(get_mcp_tools))
        .route("/api/skills", get(get_skills))
        .route("/api/password/change", post(change_password))
        .route("/api/execute/{command}", post(execute_command))
        .route("/api/cdp-ports", get(get_cdp_ports).put(update_cdp_ports))
        .route("/api/cdp-ports/refresh", post(refresh_cdp_ports))
        .route("/api/accounts", get(get_accounts))
        .fallback(crate::embedded::serve_static)
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn bootstrap(State(state): State<Arc<AppState>>) -> Json<Value> {
    let runtime = state.runtime.read().await;
    let cdp_ports = state.cdp_ports.read().await;
    let accounts = state.db.list_accounts().unwrap_or_default();
    let online_count = accounts.iter().filter(|a| a.online).count();
    let offline_count = accounts.len() - online_count;
    Json(json!({
        "first_run": state.first_run,
        "password_required": !runtime.config.auth.password.is_empty() && !runtime.config.auth.password_changed,
        "server": {
            "host": runtime.config.server.host,
            "port": runtime.config.server.port,
        },
        "agent_browser": {
            "binary": runtime.config.agent_browser.binary,
            "detected": runtime.config.agent_browser.binary != "agent-browser",
        },
        "cdp": {
            "ports": *cdp_ports,
            "online": online_count,
            "offline": offline_count,
        },
        "vnc": {
            "configured": !runtime.config.vnc.url.is_empty(),
        }
    }))
}

#[derive(Deserialize)]
struct PasswordRequest {
    password: String,
}

async fn setup_password(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PasswordRequest>,
) -> Result<Json<Value>, AppError> {
    if body.password.is_empty() {
        return Err(AppError::InvalidParams("password cannot be empty".to_string()));
    }

    let mut runtime = state.runtime.write().await;
    runtime.config.auth.password = body.password.clone();
    runtime.config.auth.password_changed = true;
    runtime.auth_state.password = body.password;
    runtime.auth_state.password_initialized = true;

    let path = config::config_path()?;
    config::save(&path, &runtime.config).await?;

    Ok(Json(json!({ "ok": true })))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let runtime = state.runtime.read().await;
    if body.password != runtime.auth_state.password {
        return Err(AppError::InvalidPassword);
    }

    let cookie = format!(
        "{AUTH_COOKIE_NAME}={}; Path=/; HttpOnly; SameSite=Strict",
        body.password
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|err| AppError::Internal(err.to_string()))?,
    );
    Ok((headers, Json(json!({ "ok": true }))))
}

async fn logout() -> impl IntoResponse {
    let cookie = format!("{AUTH_COOKIE_NAME}=; Path=/; Max-Age=0");
    let mut headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&cookie) {
        headers.insert(header::SET_COOKIE, v);
    }
    (headers, Redirect::to("/"))
}

async fn logout_api() -> impl IntoResponse {
    let cookie = format!("{AUTH_COOKIE_NAME}=; Path=/; Max-Age=0");
    let mut headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&cookie) {
        headers.insert(header::SET_COOKIE, v);
    }
    (headers, Json(json!({ "ok": true })))
}

async fn get_config(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let runtime = state.runtime.read().await;
    if !is_authenticated(&headers, &runtime.auth_state) {
        return Err(AppError::AuthRequired);
    }
    Ok(Json(json!({
        "server": runtime.config.server,
        "agent_browser": runtime.config.agent_browser,
        "vnc": { "url": runtime.config.vnc.url, "embed": runtime.config.vnc.embed },
    })))
}

async fn update_config(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let mut runtime = state.runtime.write().await;
    if !is_authenticated(&headers, &runtime.auth_state) {
        return Err(AppError::AuthRequired);
    }

    if let Some(ab) = body.get("agent_browser") {
        if let Some(binary) = ab.get("binary").and_then(Value::as_str) {
            runtime.config.agent_browser.binary = binary.to_string();
        }
        if let Some(session) = ab.get("session_name").and_then(Value::as_str) {
            runtime.config.agent_browser.session_name = session.to_string();
        }
        if let Some(timeout) = ab.get("timeout_secs").and_then(Value::as_u64) {
            runtime.config.agent_browser.timeout_secs = timeout;
        }
    }
    if let Some(vnc) = body.get("vnc") {
        if let Some(url) = vnc.get("url").and_then(Value::as_str) {
            runtime.config.vnc.url = url.to_string();
        }
        if let Some(embed) = vnc.get("embed").and_then(Value::as_bool) {
            runtime.config.vnc.embed = embed;
        }
    }

    let path = config::config_path()?;
    config::save(&path, &runtime.config).await?;

    Ok(Json(json!({ "ok": true })))
}

async fn get_commands(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!(state.manifest.commands))
}

async fn get_history(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let runtime = state.runtime.read().await;
    if !is_authenticated(&headers, &runtime.auth_state) {
        return Err(AppError::AuthRequired);
    }
    Ok(Json(json!(runtime.recent_executions)))
}

async fn get_mcp_tools(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!(state.manifest.mcp_tools))
}

async fn get_skills(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!(state.manifest.skills))
}

async fn change_password(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PasswordRequest>,
) -> Result<Json<Value>, AppError> {
    let mut runtime = state.runtime.write().await;
    if !is_authenticated(&headers, &runtime.auth_state) {
        return Err(AppError::AuthRequired);
    }
    if body.password.is_empty() {
        return Err(AppError::InvalidParams("password cannot be empty".to_string()));
    }

    runtime.config.auth.password = body.password.clone();
    runtime.config.auth.password_changed = true;
    runtime.auth_state.password = body.password;
    runtime.auth_state.password_initialized = true;

    let path = config::config_path()?;
    config::save(&path, &runtime.config).await?;

    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct ExecuteRequest {
    #[serde(default)]
    params: Value,
}

async fn execute_command(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Path(command): Path<String>,
    Json(body): Json<ExecuteRequest>,
) -> Result<Json<Value>, AppError> {
    let runtime = state.runtime.read().await;
    if !is_authenticated(&headers, &runtime.auth_state) {
        return Err(AppError::AuthRequired);
    }
    let config = runtime.config.clone();
    drop(runtime);

    let managed_ports = state.cdp_ports.read().await.clone();
    let result = state
        .executor
        .execute(&command, body.params, &config, &managed_ports)
        .await;

    let (ok, summary) = match &result {
        Ok(_) => (true, "ok".to_string()),
        Err(err) => (false, err.to_string()),
    };

    let record = ExecutionRecord::new("api", &command, ok, &summary);
    {
        let mut runtime = state.runtime.write().await;
        runtime.recent_executions.push(record);
        if runtime.recent_executions.len() > 100 {
            runtime.recent_executions.remove(0);
        }
    }

    match result {
        Ok(value) => Ok(Json(json!(
            ApiResponse::success(value, Some(command))
        ))),
        Err(err) => Err(err),
    }
}

async fn get_cdp_ports(State(state): State<Arc<AppState>>) -> Json<Value> {
    let ports = state.cdp_ports.read().await;
    Json(json!({ "ports": *ports }))
}

#[derive(Deserialize)]
struct UpdateCdpPortsRequest {
    ports: Vec<String>,
}

async fn update_cdp_ports(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateCdpPortsRequest>,
) -> Result<Json<Value>, AppError> {
    let runtime = state.runtime.read().await;
    if !is_authenticated(&headers, &runtime.auth_state) {
        return Err(AppError::AuthRequired);
    }
    drop(runtime);

    let mut ports = state.cdp_ports.write().await;
    *ports = body.ports.clone();
    drop(ports);

    let mut runtime = state.runtime.write().await;
    runtime.config.cdp_ports = body.ports;
    let path = config::config_path()?;
    config::save(&path, &runtime.config).await?;

    Ok(Json(json!({ "ok": true })))
}

async fn refresh_cdp_ports(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let runtime = state.runtime.read().await;
    if !is_authenticated(&headers, &runtime.auth_state) {
        return Err(AppError::AuthRequired);
    }
    let binary = runtime.config.agent_browser.binary.clone();
    let timeout = runtime.config.agent_browser.timeout_secs;
    drop(runtime);

    let ports = state.cdp_ports.read().await.clone();
    let db = state.db.clone();
    tokio::spawn(async move {
        discovery::discover(&db, &binary, &ports, timeout, false).await;
    });

    Ok(Json(json!({ "ok": true, "message": "refresh started" })))
}

async fn get_accounts(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let runtime = state.runtime.read().await;
    if !is_authenticated(&headers, &runtime.auth_state) {
        return Err(AppError::AuthRequired);
    }
    drop(runtime);

    let accounts = state.db.list_accounts()?;
    Ok(Json(json!(accounts)))
}
