use std::sync::{Arc, Mutex, PoisonError};

use anyhow::Result;

use crate::sqlite::Sqlite;

/// The main database shared by all requests.
///
/// SQLite calls block (a busy database is waited for and retried), so each
/// call runs on the blocking thread pool. The lock lives only inside a call
/// and therefore can never be held across an await.
#[derive(Clone)]
pub struct Database {
    sqlite: Arc<Mutex<Sqlite>>,
}

impl Database {
    #[must_use]
    pub fn new(sqlite: Sqlite) -> Self {
        Self {
            sqlite: Arc::new(Mutex::new(sqlite)),
        }
    }

    /// Runs `action` with exclusive access to the database.
    pub async fn call<T, F>(&self, action: F) -> Result<T>
    where
        F: FnOnce(&mut Sqlite) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let sqlite = Arc::clone(&self.sqlite);
        tokio::task::spawn_blocking(move || {
            // A poisoned lock still guards a usable connection: an unwinding
            // transaction is rolled back when dropped.
            let mut sqlite = sqlite.lock().unwrap_or_else(PoisonError::into_inner);
            action(&mut sqlite)
        })
        .await?
    }
}
