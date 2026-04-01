use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde::Serialize;

use crate::config;
use crate::errors::{AppError, AppResult};

const DB_FILE_NAME: &str = "data.db";

#[derive(Debug, Clone, Serialize)]
pub struct AccountEntry {
    pub cdp_port: String,
    pub email: String,
    pub display_name: String,
    pub online: bool,
    pub last_checked: u64,
}

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

fn db_path() -> AppResult<PathBuf> {
    Ok(config::config_dir()?.join(DB_FILE_NAME))
}

impl Db {
    fn init(conn: Connection) -> AppResult<Self> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS accounts (
                cdp_port     TEXT PRIMARY KEY,
                email        TEXT NOT NULL DEFAULT '',
                display_name TEXT NOT NULL DEFAULT '',
                online       INTEGER NOT NULL DEFAULT 0,
                last_checked INTEGER NOT NULL DEFAULT 0
            );",
        )
        .map_err(|err| AppError::Internal(err.to_string()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open() -> AppResult<Self> {
        let path = db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| AppError::Internal(err.to_string()))?;
        }
        let conn =
            Connection::open(&path).map_err(|err| AppError::Internal(err.to_string()))?;
        Self::init(conn)
    }

    pub fn list_accounts(&self) -> AppResult<Vec<AccountEntry>> {
        let conn = self.conn.lock().map_err(|err| AppError::Internal(err.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT cdp_port, email, display_name, online, last_checked FROM accounts ORDER BY cdp_port",
            )
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(AccountEntry {
                    cdp_port: row.get(0)?,
                    email: row.get(1)?,
                    display_name: row.get(2)?,
                    online: row.get::<_, i32>(3)? != 0,
                    last_checked: row.get(4)?,
                })
            })
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|err| AppError::Internal(err.to_string()))?);
        }
        Ok(result)
    }

    pub fn upsert_account(&self, entry: &AccountEntry) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|err| AppError::Internal(err.to_string()))?;
        conn.execute(
            "INSERT INTO accounts (cdp_port, email, display_name, online, last_checked)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(cdp_port) DO UPDATE SET
                email = excluded.email,
                display_name = excluded.display_name,
                online = excluded.online,
                last_checked = excluded.last_checked",
            rusqlite::params![
                entry.cdp_port,
                entry.email,
                entry.display_name,
                entry.online as i32,
                entry.last_checked,
            ],
        )
        .map_err(|err| AppError::Internal(err.to_string()))?;
        Ok(())
    }

    pub fn get_account(&self, cdp_port: &str) -> AppResult<Option<AccountEntry>> {
        let conn = self.conn.lock().map_err(|err| AppError::Internal(err.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT cdp_port, email, display_name, online, last_checked FROM accounts WHERE cdp_port = ?1")
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let mut rows = stmt
            .query_map(rusqlite::params![cdp_port], |row| {
                Ok(AccountEntry {
                    cdp_port: row.get(0)?,
                    email: row.get(1)?,
                    display_name: row.get(2)?,
                    online: row.get::<_, i32>(3)? != 0,
                    last_checked: row.get(4)?,
                })
            })
            .map_err(|err| AppError::Internal(err.to_string()))?;
        match rows.next() {
            Some(Ok(entry)) => Ok(Some(entry)),
            Some(Err(err)) => Err(AppError::Internal(err.to_string())),
            None => Ok(None),
        }
    }

    pub fn ensure_port(&self, cdp_port: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|err| AppError::Internal(err.to_string()))?;
        conn.execute(
            "INSERT OR IGNORE INTO accounts (cdp_port) VALUES (?1)",
            rusqlite::params![cdp_port],
        )
        .map_err(|err| AppError::Internal(err.to_string()))?;
        Ok(())
    }

    pub fn set_offline(&self, cdp_port: &str, timestamp: u64) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|err| AppError::Internal(err.to_string()))?;
        conn.execute(
            "UPDATE accounts SET online = 0, last_checked = ?2 WHERE cdp_port = ?1",
            rusqlite::params![cdp_port, timestamp],
        )
        .map_err(|err| AppError::Internal(err.to_string()))?;
        Ok(())
    }
}
