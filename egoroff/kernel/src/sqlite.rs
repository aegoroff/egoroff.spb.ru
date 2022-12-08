use std::path::Path;

use rusqlite::{params, Connection, Error, ErrorCode, OpenFlags, Transaction};

use crate::domain::{Post, Storage};

pub enum Mode {
    ReadWrite,
    ReadOnly,
}

pub struct Sqlite {
    conn: Connection,
}

impl Storage for Sqlite {
    type Err = Error;

    fn new_database(&self) -> Result<(), Self::Err> {
        self.pragma_update("encoding", "UTF-8")?;

        self.conn.execute(
            "CREATE TABLE post (
                  id              INTEGER PRIMARY KEY,
                  title           TEXT NOT NULL,
                  short_text      TEXT NOT NULL,
                  text            TEXT NOT NULL,
                  markdown        INTEGER,
                  is_public       INTEGER,
                  created         INTEGER NOT NULL
                  modified        INTEGER NOT NULL
                  )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX created_ix ON post(created)",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE post_tag (
                  post_id              INTEGER NOT NULL,
                  tag                  TEXT NOT NULL,
                  PRIMARY KEY (post_id, tag)
                  )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE tag (
                  title           TEXT PRIMARY KEY
                  )",
            [],
        )?;

        Ok(())
    }

    fn get_small_posts(
        &self,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<crate::domain::SmallPost>, Self::Err> {
        todo!()
    }

    fn get_post(&self, _id: i64) -> Result<crate::domain::Post, Self::Err> {
        todo!()
    }

    fn upsert_post(&mut self, post: crate::domain::Post) -> Result<(), Self::Err> {
        let items = vec![post];
        self.upsert(items, Sqlite::upsert_post)?;
        Ok(())
    }

    fn delete_post(&mut self, _id: i64) -> Result<(), Self::Err> {
        todo!()
    }
}

impl Sqlite {
    pub fn open<P: AsRef<Path>>(path: P, mode: Mode) -> Result<impl Storage, Error> {
        let c = match mode {
            Mode::ReadWrite => Connection::open(path),
            Mode::ReadOnly => Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY),
        };
        Ok(Self { conn: c? })
    }

    fn upsert_post(tx: &Transaction, p: &Post) -> usize {
        tx.prepare_cached(
            "INSERT INTO post (id, title, short_text, text, created, modified, is_public, markdown) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT(id) DO UPDATE SET title=?2, short_text=?3, text=?4, created=?5, modified=?6, is_piblic=?7, markdown=?8",
        )
        .unwrap()
        .execute(params![p.id, p.title, p.short_text, p.text, p.created.timestamp(), p.modified.timestamp(), p.is_public, p.markdown])
        .unwrap_or_default()
    }

    fn upsert<T>(
        &mut self,
        items: Vec<T>,
        fn_execute: fn(&Transaction, &T) -> usize,
    ) -> Result<usize, Error> {
        Sqlite::execute_with_retry(|| {
            let mut result: usize = 0;
            let tx = self.conn.transaction()?;
            for item in &items {
                let res = fn_execute(&tx, item);
                result += res;
            }
            tx.commit()?;

            Ok(result)
        })
    }

    fn pragma_update(&self, name: &str, value: &str) -> Result<(), Error> {
        self.conn.pragma_update(None, name, value)
    }

    fn execute_with_retry<T, F>(mut action: F) -> Result<T, Error>
    where
        F: FnMut() -> Result<T, Error>,
    {
        loop {
            let result = action();
            if let Err(err) = result {
                if let Error::SqliteFailure(e, _) = err {
                    if e.code == ErrorCode::DatabaseBusy {
                        continue;
                    } else {
                        return Err(err);
                    }
                } else {
                    return Err(err);
                }
            } else {
                return result;
            }
        }
    }
}
