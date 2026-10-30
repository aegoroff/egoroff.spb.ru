#![allow(clippy::unwrap_used)]

use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use rusqlite::Connection;

/// Long enough to tell a stalled runtime from a busy one.
pub const LOCK_TIME: Duration = Duration::from_millis(500);

/// A database file in the temp dir unique to this test process.
pub fn temp_db(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("egoroff-{name}-{}.db", std::process::id()))
}

/// Runs `action` while another connection holds an exclusive lock on `path`
/// for [`LOCK_TIME`] and returns how long the async runtime could not run
/// other tasks, together with the action's result.
pub async fn stall_while_locked<F>(path: &Path, action: F) -> (Duration, F::Output)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let (locked_tx, locked_rx) = mpsc::channel();
    let blocker_path = path.to_owned();
    let blocker = thread::spawn(move || {
        let conn = Connection::open(blocker_path).unwrap();
        conn.execute_batch("BEGIN EXCLUSIVE").unwrap();
        locked_tx.send(()).unwrap();
        thread::sleep(LOCK_TIME);
        conn.execute_batch("ROLLBACK").unwrap();
    });
    locked_rx.recv().unwrap();

    let task = tokio::spawn(action);
    let started = Instant::now();
    tokio::time::sleep(Duration::from_millis(10)).await;
    let stall = started.elapsed();
    let result = task.await.unwrap();
    blocker.join().unwrap();
    (stall, result)
}
