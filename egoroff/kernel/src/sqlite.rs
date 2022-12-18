use std::path::Path;

use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{params, Connection, Error, ErrorCode, OpenFlags, Row, Transaction};

use crate::domain::{OAuthProvider, Post, PostsRequest, SmallPost, Storage, TagAggregate, User};

pub enum Mode {
    ReadWrite,
    ReadOnly,
}

pub const DATABASE: &str = "egoroff.db";

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
                  created         INTEGER NOT NULL,
                  modified        INTEGER NOT NULL
                  )",
            [],
        )?;

        self.conn
            .execute("CREATE INDEX created_ix ON post(created)", [])?;

        self.conn.execute(
            "CREATE TABLE tag (
                   tag           TEXT PRIMARY KEY
                  )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE post_tag (
                  post_id              INTEGER NOT NULL,
                  tag                  TEXT NOT NULL,
                  PRIMARY KEY (post_id, tag)
                  FOREIGN KEY(post_id) REFERENCES post(id)
                  FOREIGN KEY(tag) REFERENCES tag(tag)
                  )",
            [],
        )?;

        Ok(())
    }

    fn get_small_posts(
        &self,
        limit: i32,
        offset: i32,
        request: PostsRequest,
    ) -> Result<Vec<crate::domain::SmallPost>, Self::Err> {
        self.enable_foreign_keys()?;

        let files: Vec<crate::domain::SmallPost> = match request.tag {
            Some(tag) => {
                let mut stmt = self.conn.prepare("SELECT id, title, created, short_text, markdown \
                                    FROM post INNER JOIN post_tag ON post_tag.post_id = post.id 
                                    WHERE is_public = 1 AND post_tag.tag = ?3 ORDER BY created DESC LIMIT ?1 OFFSET ?2")?;
                let files = stmt.query_map(
                    [limit.to_string(), offset.to_string(), tag],
                    Sqlite::map_small_post_row,
                )?;
                files.filter_map(|r| r.ok()).collect()
            }
            None => {
                if let Some(period) = request.as_query_period() {
                    let mut stmt = self.conn.prepare(
                        "SELECT id, title, created, short_text, markdown \
                    FROM post WHERE is_public = 1 AND created > ?1 AND created < ?2 ORDER BY created DESC  LIMIT ?3 OFFSET ?4",
                    )?;
                    let files = stmt.query_map(
                        [
                            period.from.timestamp(),
                            period.to.timestamp(),
                            limit as i64,
                            offset as i64,
                        ],
                        Sqlite::map_small_post_row,
                    )?;
                    files.filter_map(|r| r.ok()).collect()
                } else {
                    let mut stmt = self.conn.prepare(
                        "SELECT id, title, created, short_text, markdown \
                    FROM post WHERE is_public = 1 ORDER BY created DESC LIMIT ?1 OFFSET ?2",
                    )?;
                    let files = stmt.query_map([limit, offset], Sqlite::map_small_post_row)?;
                    files.filter_map(|r| r.ok()).collect()
                }
            }
        };
        Ok(files)
    }

    fn get_post(&self, id: i64) -> Result<crate::domain::Post, Self::Err> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag FROM post_tag WHERE post_id = ?1")?;
        let tags = stmt.query_map([id], |row| {
            let tag = row.get(0)?;
            Ok(tag)
        })?;

        let mut stmt = self
            .conn
            .prepare("SELECT title, created, short_text, markdown, text, is_public, modified FROM post WHERE id=?1")?;
        let post: Post = stmt.query_row([id], |row| {
            let created: i64 = row.get(1)?;
            let modified: i64 = row.get(6)?;
            let created_datetime =
                NaiveDateTime::from_timestamp_opt(created, 0).unwrap_or(NaiveDateTime::MIN);
            let modified_datetime =
                NaiveDateTime::from_timestamp_opt(modified, 0).unwrap_or(NaiveDateTime::MIN);
            let created = DateTime::<Utc>::from_utc(created_datetime, Utc);
            let modified = DateTime::<Utc>::from_utc(modified_datetime, Utc);

            let post = Post {
                created,
                modified,
                id,
                title: row.get(0)?,
                short_text: row.get(2)?,
                text: row.get(4)?,
                markdown: row.get(3)?,
                is_public: row.get(5)?,
                tags: tags.filter_map(|r| r.ok()).collect(),
            };

            Ok(post)
        })?;

        Ok(post)
    }

    fn upsert_post(&mut self, post: crate::domain::Post) -> Result<(), Self::Err> {
        let items = vec![post];
        self.upsert(items, Sqlite::upsert_post)?;
        Ok(())
    }

    fn delete_post(&mut self, _id: i64) -> Result<(), Self::Err> {
        todo!()
    }

    fn count_posts(&self, request: PostsRequest) -> Result<i32, Self::Err> {
        match request.tag {
            Some(tag) => {
                let mut stmt = self.conn.prepare(
                    "SELECT COUNT(1) FROM post INNER JOIN post_tag ON post_tag.post_id = post.id 
                    WHERE is_public = 1 AND post_tag.tag = ?1",
                )?;
                stmt.query_row([tag], |row| row.get(0))
            }
            None => {
                if let Some(period) = request.as_query_period() {
                    let mut stmt = self.conn.prepare(
                        "SELECT COUNT(1) FROM post WHERE is_public = 1 AND created > ?1 AND created < ?2",
                    )?;
                    stmt.query_row([period.from.timestamp(), period.to.timestamp()], |row| {
                        row.get(0)
                    })
                } else {
                    let mut stmt = self
                        .conn
                        .prepare("SELECT COUNT(1) FROM post WHERE is_public = 1")?;
                    stmt.query_row([], |row| row.get(0))
                }
            }
        }
    }

    fn get_aggregate_tags(&self) -> Result<Vec<crate::domain::TagAggregate>, Self::Err> {
        self.enable_foreign_keys()?;

        let mut stmt = self.conn.prepare("SELECT post_tag.tag, count(1) FROM post_tag \
                                                            INNER JOIN post ON post_tag.post_id = post.id WHERE post.is_public = 1 GROUP BY tag")?;
        let files = stmt.query_map([], |row| {
            let tag = TagAggregate {
                title: row.get(0)?,
                count: row.get(1)?,
            };
            Ok(tag)
        })?;

        Ok(files.filter_map(|r| r.ok()).collect())
    }

    fn get_posts_create_dates(&self) -> Result<Vec<DateTime<Utc>>, Self::Err> {
        let mut stmt = self
            .conn
            .prepare("SELECT created FROM post WHERE is_public = 1 ORDER BY created DESC")?;
        let dates = stmt.query_map([], |row| {
            let created: i64 = row.get(0)?;
            let created_datetime =
                NaiveDateTime::from_timestamp_opt(created, 0).unwrap_or(NaiveDateTime::MIN);
            Ok(DateTime::<Utc>::from_utc(created_datetime, Utc))
        })?;
        Ok(dates.filter_map(|r| r.ok()).collect())
    }

    fn get_posts_ids(&self) -> Result<Vec<i64>, Self::Err> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM post WHERE is_public = 1 ORDER BY created DESC")?;
        let ids = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            Ok(id)
        })?;
        Ok(ids.filter_map(|r| r.ok()).collect())
    }

    fn get_oauth_provider(&self, name: &str) -> Result<crate::domain::OAuthProvider, Self::Err> {
        let mut stmt = self
            .conn
            .prepare("SELECT scope FROM oauth_provider_scopes WHERE provider = ?1")?;
        let scopes = stmt.query_map([name], |row| {
            let tag = row.get(0)?;
            Ok(tag)
        })?;

        let mut stmt = self.conn.prepare(
            "SELECT name, clientid, secret, redirect_url FROM oauth_provider WHERE name=?1",
        )?;
        let provider: OAuthProvider = stmt.query_row([name], |row| {
            let provider = OAuthProvider {
                name: row.get(0)?,
                client_id: row.get(1)?,
                secret: row.get(2)?,
                redirect_url: row.get(3)?,
                scopes: scopes.filter_map(|r| r.ok()).collect(),
            };

            Ok(provider)
        })?;

        Ok(provider)
    }

    fn get_user(&self, federated_id: &str, provider: &str) -> Result<crate::domain::User, Self::Err> {
        let mut stmt = self
            .conn
            .prepare("SELECT created, email, name, login, avatar_url, federated_id, admin, verified, provider FROM user WHERE federated_id=?1 AND provider=?2")?;
        let user: User = stmt.query_row([federated_id, provider], |row| {
            let created: i64 = row.get(0)?;
            let created_datetime =
                NaiveDateTime::from_timestamp_opt(created, 0).unwrap_or(NaiveDateTime::MIN);
            let created = DateTime::<Utc>::from_utc(created_datetime, Utc);

            let user = User {
                created,
                email: row.get(1)?,
                name: row.get(2)?,
                login: row.get(3)?,
                avatar_url: row.get(4)?,
                federated_id: row.get(5)?,
                admin: row.get(6)?,
                verified: row.get(7)?,
                provider: row.get(8)?,
            };

            Ok(user)
        })?;

        Ok(user)
    }

    fn upsert_user(&mut self, user: &crate::domain::User) -> Result<(), Self::Err> {
        
        Sqlite::execute_with_retry(|| {
            let tx = self.conn.transaction()?;
            Sqlite::upsert_user(&tx, user);
            tx.commit()?;

            Ok(())
        })?;

        Ok(())
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

    fn map_small_post_row<E: std::convert::From<Error>>(row: &Row<'_>) -> Result<SmallPost, E> {
        let created: i64 = row.get(2)?;
        let datetime = NaiveDateTime::from_timestamp_opt(created, 0).unwrap_or(NaiveDateTime::MIN);
        let created = DateTime::<Utc>::from_utc(datetime, Utc);

        let post = SmallPost {
            id: row.get(0)?,
            title: row.get(1)?,
            created,
            short_text: row.get(3)?,
            markdown: row.get(4)?,
        };
        Ok(post)
    }

    fn upsert_post(tx: &Transaction, p: &Post) -> usize {
        let result = tx.prepare_cached(
            "INSERT INTO post (id, title, short_text, text, created, modified, is_public, markdown) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT(id) DO UPDATE SET title=?2, short_text=?3, text=?4, created=?5, modified=?6, is_public=?7, markdown=?8",
        )
        .unwrap()
        .execute(params![p.id, p.title, p.short_text, p.text, p.created.timestamp(), p.modified.timestamp(), p.is_public, p.markdown])
        .unwrap_or_default();

        let mut tag_statement = tx
            .prepare_cached(
                "INSERT INTO tag (tag) VALUES (?1)
                ON CONFLICT(tag) DO UPDATE SET tag=?1",
            )
            .unwrap();

        let mut post_tag_statement = tx
            .prepare_cached(
                "INSERT INTO post_tag (post_id, tag) VALUES (?1, ?2)
                ON CONFLICT(post_id, tag) DO UPDATE SET post_id=?1, tag=?2",
            )
            .unwrap();

        for t in p.tags.iter() {
            tag_statement.execute(params![t]).unwrap_or_default();
            post_tag_statement
                .execute(params![p.id, t])
                .unwrap_or_default();
        }

        result
    }

    fn upsert_user(tx: &Transaction, u: &User) -> usize {
        let result = tx.prepare_cached(
            "INSERT INTO user (created, email, name, login, avatar_url, federated_id, admin, verified, provider) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(federated_id, provider) DO UPDATE SET email=?2, name=?3, login=?4, avatar_url=?5, admin=?7, verified=?8",
        )
        .unwrap()
        .execute(params![u.created.timestamp(), u.email, u.name, u.login, u.avatar_url, u.federated_id, u.admin, u.verified, u.provider])
        .unwrap_or_default();

        result
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

    fn enable_foreign_keys(&self) -> Result<(), Error> {
        self.pragma_update("foreign_keys", "ON")
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
