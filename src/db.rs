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

#[derive(Debug, Clone, Serialize)]
pub struct PreviewNoteEntry {
    pub id: i64,
    pub cdp_port: String,
    pub google_account: String,
    pub notebook_id: String,
    pub notebook_title: String,
    pub note_key: String,
    pub note_title: String,
    pub content: String,
    pub content_preview: String,
    pub fetched_at: u64,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct NewPreviewNoteEntry {
    pub cdp_port: String,
    pub google_account: String,
    pub notebook_id: String,
    pub notebook_title: String,
    pub note_key: String,
    pub note_title: String,
    pub content: String,
    pub content_preview: String,
    pub fetched_at: u64,
    pub created_at: u64,
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
            );

            CREATE TABLE IF NOT EXISTS preview_notes (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                cdp_port        TEXT NOT NULL,
                google_account  TEXT NOT NULL DEFAULT '',
                notebook_id     TEXT NOT NULL,
                notebook_title  TEXT NOT NULL DEFAULT '',
                note_key        TEXT NOT NULL,
                note_title      TEXT NOT NULL,
                content         TEXT NOT NULL DEFAULT '',
                content_preview TEXT NOT NULL DEFAULT '',
                fetched_at      INTEGER NOT NULL DEFAULT 0,
                created_at      INTEGER NOT NULL DEFAULT 0,
                UNIQUE(cdp_port, notebook_id, note_key)
            );

            CREATE INDEX IF NOT EXISTS idx_preview_notes_fetched_at
                ON preview_notes (fetched_at DESC);
            CREATE INDEX IF NOT EXISTS idx_preview_notes_account
                ON preview_notes (google_account, fetched_at DESC);",
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

    pub fn list_preview_notes(&self) -> AppResult<Vec<PreviewNoteEntry>> {
        let conn = self.conn.lock().map_err(|err| AppError::Internal(err.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, cdp_port, google_account, notebook_id, notebook_title, note_key,
                        note_title, content, content_preview, fetched_at, created_at
                 FROM preview_notes
                 ORDER BY fetched_at DESC, id DESC",
            )
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PreviewNoteEntry {
                    id: row.get(0)?,
                    cdp_port: row.get(1)?,
                    google_account: row.get(2)?,
                    notebook_id: row.get(3)?,
                    notebook_title: row.get(4)?,
                    note_key: row.get(5)?,
                    note_title: row.get(6)?,
                    content: row.get(7)?,
                    content_preview: row.get(8)?,
                    fetched_at: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|err| AppError::Internal(err.to_string()))?);
        }
        Ok(result)
    }

    pub fn preview_note_exists(
        &self,
        google_account: &str,
        notebook_id: &str,
        note_key: &str,
    ) -> AppResult<bool> {
        let conn = self.conn.lock().map_err(|err| AppError::Internal(err.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT 1 FROM preview_notes
                 WHERE google_account = ?1 AND notebook_id = ?2 AND note_key = ?3
                 LIMIT 1",
            )
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let mut rows = stmt
            .query(rusqlite::params![google_account, notebook_id, note_key])
            .map_err(|err| AppError::Internal(err.to_string()))?;
        Ok(rows
            .next()
            .map_err(|err| AppError::Internal(err.to_string()))?
            .is_some())
    }

    pub fn insert_preview_note(&self, entry: &NewPreviewNoteEntry) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|err| AppError::Internal(err.to_string()))?;
        conn.execute(
            "INSERT OR IGNORE INTO preview_notes (
                cdp_port, google_account, notebook_id, notebook_title, note_key,
                note_title, content, content_preview, fetched_at, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                entry.cdp_port,
                entry.google_account,
                entry.notebook_id,
                entry.notebook_title,
                entry.note_key,
                entry.note_title,
                entry.content,
                entry.content_preview,
                entry.fetched_at,
                entry.created_at,
            ],
        )
        .map_err(|err| AppError::Internal(err.to_string()))?;
        Ok(())
    }
}
