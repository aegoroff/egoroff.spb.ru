use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rusqlite::{params, Connection};

use tower_sessions::{session::Id, Session, SessionStore};

#[derive(Debug, Clone)]
pub struct SqliteSessionStore {
    path: Arc<PathBuf>,
}

impl SqliteSessionStore {
    pub fn open(path: PathBuf, secret: &[u8]) -> Result<SqliteSessionStore, rusqlite::Error> {
        let conn = SqliteSessionStore::create_connection(&path)?;
        conn.execute(
            r#"
                    CREATE TABLE IF NOT EXISTS session (
                        id TEXT PRIMARY KEY NOT NULL,
                        expires INTEGER NULL,
                        session BLOB NOT NULL
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

    pub fn cleanup(&self) -> Result<(), rusqlite::Error> {
        let conn = SqliteSessionStore::create_connection(&self.path)?;
        let mut stmt = conn.prepare(
            r#"
            DELETE FROM session WHERE expires < ?1
            "#,
        )?;

        stmt.execute(params![Utc::now().timestamp()])?;

        Ok(())
    }

    pub fn get_secret(&self) -> Result<Vec<u8>, rusqlite::Error> {
        let conn = SqliteSessionStore::create_connection(&self.path)?;
        let mut stmt = conn.prepare("SELECT secret FROM secret")?;
        let encoded: String = stmt.query_row([], |row| {
            let s: String = row.get(0)?;
            Ok(s)
        })?;

        let result = general_purpose::STANDARD
            .decode(encoded)
            .unwrap_or_default();

        Ok(result)
    }

    fn create_connection(path: &Path) -> Result<Connection, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "synchronous", "OFF")?;
        conn.pragma_update(None, "journal_mode", "MEMORY")?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        Ok(conn)
    }
}

#[async_trait]
impl SessionStore for SqliteSessionStore {
    type Error = rusqlite::Error;

    async fn load(&self, session_id: &Id) -> Result<Option<Session>, Self::Error> {
        let id = session_id.to_string();
        let conn = SqliteSessionStore::create_connection(&self.path)?;

        let mut stmt = conn.prepare(
            r#"
            SELECT session, expires, id FROM session
              WHERE id = ?1 AND (expires IS NULL OR expires > ?2)
            "#,
        )?;

        let now = Utc::now().timestamp();
        let parameters = params![id, now];
        let record = stmt.query_row(parameters, |row| {
            let data: Vec<u8> = row.get(0)?;
            let data: Session = rmp_serde::from_slice(&data).unwrap_or_default();
            Ok(data)
        })?;

        Ok(Some(record))
    }

    async fn save(&self, session: &Session) -> Result<(), Self::Error> {
        let id = session.id().to_string();
        let data = rmp_serde::to_vec(&session).unwrap_or_default();
        let expiry = &session.expiry_date().unix_timestamp();

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
        let parameters = params![id, data, expiry];
        stmt.execute(parameters)?;
        Ok(())
    }

    async fn delete(&self, session_id: &Id) -> Result<(), Self::Error> {
        let id = session_id.to_string();
        let conn = SqliteSessionStore::create_connection(&self.path)?;
        let mut stmt = conn.prepare("DELETE FROM session WHERE id = ?")?;

        stmt.execute(params![id])?;

        Ok(())
    }
}
