use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use rusqlite::{Connection, params};

use tower_sessions::session_store::{Error, Result};
use tower_sessions::{
    SessionStore,
    session::{Id, Record},
};

#[derive(Debug, Clone)]
pub struct SqliteSessionStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteSessionStore {
    pub fn open(path: &Path, secret: &[u8]) -> anyhow::Result<SqliteSessionStore> {
        let conn = SqliteSessionStore::create_connection(path)?;
        conn.execute(
            r"
                    CREATE TABLE IF NOT EXISTS session (
                        id TEXT PRIMARY KEY NOT NULL,
                        expires INTEGER NULL,
                        session BLOB NOT NULL
                    )
                 ",
            [],
        )?;

        conn.execute(
            r"
                    CREATE TABLE IF NOT EXISTS secret (
                        secret TEXT PRIMARY KEY NOT NULL
                    )
                 ",
            [],
        )?;

        let mut stmt = conn.prepare("SELECT COUNT(1) FROM secret")?;
        let secret_count: i32 = stmt.query_row([], |row| row.get(0))?;
        if secret_count == 0 {
            let mut stmt = conn.prepare(
                r"
                INSERT INTO secret
                  (secret) VALUES (?1)
                ",
            )?;
            let secret = general_purpose::STANDARD.encode(secret);
            let parameters = params![secret];
            stmt.execute(parameters)?;
        }
        drop(stmt);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn cleanup(&self) -> anyhow::Result<()> {
        let conn = self.connection();
        let mut stmt = conn.prepare(r"DELETE FROM session WHERE expires < ?1")?;

        stmt.execute(params![Utc::now().timestamp()])?;

        Ok(())
    }

    pub fn get_secret(&self) -> anyhow::Result<Vec<u8>> {
        let conn = self.connection();
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

    /// A poisoned lock still guards a usable connection: every statement is atomic.
    fn connection(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn create_connection(path: &Path) -> anyhow::Result<Connection> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "synchronous", "OFF")?;
        conn.pragma_update(None, "journal_mode", "MEMORY")?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        Ok(conn)
    }

    fn load_impl(&self, session_id: &Id) -> anyhow::Result<Option<Record>> {
        let id = session_id.to_string();
        let conn = self.connection();

        let mut stmt = conn.prepare_cached(
            r"
            SELECT session, expires, id FROM session
              WHERE id = ?1 AND (expires IS NULL OR expires > ?2)
            ",
        )?;

        let now = Utc::now().timestamp();
        let parameters = params![id, now];
        let record = stmt.query_row(parameters, |row| {
            let data: Vec<u8> = row.get(0)?;
            let data = rmp_serde::from_slice(&data).ok();
            Ok(data)
        });

        match record {
            Ok(r) => Ok(r),
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                e => Err(e.into()),
            },
        }
    }

    fn delete_impl(&self, session_id: &Id) -> anyhow::Result<()> {
        let id = session_id.to_string();
        let conn = self.connection();
        let mut stmt = conn.prepare_cached("DELETE FROM session WHERE id = ?")?;

        stmt.execute(params![id])?;

        Ok(())
    }

    fn save_impl(&self, session_record: &Record) -> anyhow::Result<()> {
        let id = session_record.id.to_string();
        let data = rmp_serde::to_vec(&session_record).unwrap_or_default();
        let expiry = &session_record.expiry_date.unix_timestamp();

        let conn = self.connection();

        let mut stmt = conn.prepare_cached(
            r"
            INSERT INTO session
              (id, session, expires) VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
              expires = excluded.expires,
              session = excluded.session
            ",
        )?;
        let parameters = params![id, data, expiry];
        stmt.execute(parameters)?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for SqliteSessionStore {
    async fn load(&self, session_id: &Id) -> Result<Option<Record>> {
        self.load_impl(session_id)
            .map_err(|e| Error::Backend(e.to_string()))
    }

    async fn save(&self, session_record: &Record) -> Result<()> {
        self.save_impl(session_record)
            .map_err(|e| Error::Backend(e.to_string()))
    }

    async fn delete(&self, session_id: &Id) -> Result<()> {
        self.delete_impl(session_id)
            .map_err(|e| Error::Backend(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use rstest::{fixture, rstest};
    use tower_sessions::cookie::time::{Duration, OffsetDateTime};

    const SECRET: &[u8] = b"secret";

    #[fixture]
    fn store() -> SqliteSessionStore {
        SqliteSessionStore::open(Path::new(":memory:"), SECRET).unwrap()
    }

    fn record(expires_in: Duration) -> Record {
        Record {
            id: Id::default(),
            data: [(String::from("user"), serde_json::json!("egr"))]
                .into_iter()
                .collect(),
            expiry_date: OffsetDateTime::now_utc().replace_nanosecond(0).unwrap() + expires_in,
        }
    }

    #[rstest]
    #[case(Duration::hours(1), true)]
    #[case(Duration::hours(-1), false)]
    #[tokio::test]
    async fn load_returns_only_unexpired_saved_record(
        store: SqliteSessionStore,
        #[case] expires_in: Duration,
        #[case] expected_found: bool,
    ) {
        // arrange
        let saved = record(expires_in);
        store.save(&saved).await.unwrap();

        // act
        let loaded = store.load(&saved.id).await.unwrap();

        // assert
        assert_eq!(expected_found.then_some(saved), loaded);
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_saved_record(store: SqliteSessionStore) {
        // arrange
        let saved = record(Duration::hours(1));
        store.save(&saved).await.unwrap();

        // act
        store.delete(&saved.id).await.unwrap();

        // assert
        assert!(store.load(&saved.id).await.unwrap().is_none());
    }

    #[rstest]
    fn get_secret_returns_secret_stored_on_open(store: SqliteSessionStore) {
        // arrange

        // act
        let actual = store.get_secret().unwrap();

        // assert
        assert_eq!(SECRET, actual.as_slice());
    }
}
