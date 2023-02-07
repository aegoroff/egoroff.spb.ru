use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_session::{async_trait, serde_json, Result, Session, SessionStore};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct SqliteSessionStore {
    path: Arc<PathBuf>,
}

impl SqliteSessionStore {
    pub fn open(path: PathBuf, secret: &[u8]) -> Result<SqliteSessionStore> {
        let conn = SqliteSessionStore::create_connection(&path)?;
        conn.execute(
            r#"
                    CREATE TABLE IF NOT EXISTS session (
                        id TEXT PRIMARY KEY NOT NULL,
                        expires INTEGER NULL,
                        session TEXT NOT NULL
                    )
                 "#,
            [],
        )?;

        conn.execute(
            r#"
                    CREATE TABLE IF NOT EXISTS secret (
                        secret TEXT PRIMARY KEY NOT NULL
                    )
                 "#,
            [],
        )?;

        let mut stmt = conn.prepare("SELECT COUNT(1) FROM secret")?;
        let secret_count: i32 = stmt.query_row([], |row| row.get(0))?;
        if secret_count == 0 {
            let mut stmt = conn.prepare(
                r#"
                INSERT INTO secret
                  (secret) VALUES (?1)
                "#,
            )?;
            let secret = general_purpose::STANDARD.encode(secret);
            let parameters = params![secret];
            stmt.execute(parameters)?;
        }

        Ok(Self {
            path: Arc::new(path),
        })
    }

    pub fn cleanup(&self) -> Result<()> {
        let conn = SqliteSessionStore::create_connection(&self.path)?;
        let mut stmt = conn.prepare(
            r#"
            DELETE FROM session WHERE expires < ?1
            "#,
        )?;

        stmt.execute(params![Utc::now().timestamp()])?;

        Ok(())
    }

    pub fn get_secret(&self) -> Result<Vec<u8>> {
        let conn = SqliteSessionStore::create_connection(&self.path)?;
        let mut stmt = conn.prepare("SELECT secret FROM secret")?;
        let encoded: String = stmt.query_row([], |row| {
            let s: String = row.get(0)?;
            Ok(s)
        })?;

        let result = general_purpose::STANDARD.decode(encoded)?;

        Ok(result)
    }

    fn create_connection(path: &Path) -> Result<Connection> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "synchronous", "OFF")?;
        conn.pragma_update(None, "journal_mode", "MEMORY")?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        Ok(conn)
    }
}

#[async_trait]
impl SessionStore for SqliteSessionStore {
    async fn load_session(&self, cookie_value: String) -> Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        let conn = SqliteSessionStore::create_connection(&self.path)?;

        let mut stmt = conn.prepare(
            r#"
            SELECT session FROM session
              WHERE id = ?1 AND (expires IS NULL OR expires > ?2)
            "#,
        )?;

        let now = Utc::now().timestamp();
        let parameters = params![id, now];
        let session: Session = stmt.query_row(parameters, |row| {
            let s: String = row.get(0)?;
            let s: Session = serde_json::from_str(&s).unwrap_or_default();
            Ok(s)
        })?;

        Ok(Some(session))
    }

    async fn store_session(&self, session: Session) -> Result<Option<String>> {
        let id = session.id();
        let string = serde_json::to_string(&session)?;
        let conn = SqliteSessionStore::create_connection(&self.path)?;

        let mut stmt = conn.prepare(
            r#"
            INSERT INTO session
              (id, session, expires) VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
              expires = excluded.expires,
              session = excluded.session
            "#,
        )?;
        let expiry = &session.expiry().map(|expiry| expiry.timestamp());
        let parameters = params![id, string, expiry];
        stmt.execute(parameters)?;
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> Result {
        let id = session.id();
        let conn = SqliteSessionStore::create_connection(&self.path)?;
        let mut stmt = conn.prepare("DELETE FROM session WHERE id = ?")?;

        stmt.execute(params![id])?;

        Ok(())
    }

    async fn clear_store(&self) -> Result {
        let conn = SqliteSessionStore::create_connection(&self.path)?;
        let mut stmt = conn.prepare("DELETE FROM session")?;

        stmt.execute([])?;

        Ok(())
    }
}
