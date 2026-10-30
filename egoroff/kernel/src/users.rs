use anyhow::Result;

use crate::{
    database::Database,
    domain::{Storage, User},
};

/// Users signed in through an OAuth provider.
#[derive(Clone)]
pub struct Users {
    db: Database,
}

impl Users {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn find(&self, federated_id: String, provider: String) -> Result<User> {
        self.db
            .call(move |storage| storage.get_user(&federated_id, &provider))
            .await
    }

    /// Stores the user and hands it back.
    pub async fn save(&self, user: User) -> Result<User> {
        self.db
            .call(|storage| {
                storage.upsert_user(&user)?;
                Ok(user)
            })
            .await
    }

    /// All users, newest first.
    pub async fn all(&self) -> Result<Vec<User>> {
        self.db.call(|storage| storage.get_users()).await
    }

    pub async fn count(&self) -> Result<i32> {
        self.db.call(|storage| storage.count_users()).await
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::sqlite::{Mode, Sqlite};
    use chrono::{TimeZone, Utc};
    use rstest::{fixture, rstest};

    #[fixture]
    fn users() -> Users {
        let storage = Sqlite::open(":memory:", Mode::ReadWrite).unwrap();
        Users::new(Database::new(storage))
    }

    fn user(federated_id: &str, day: u32) -> User {
        User {
            created: Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
            email: format!("{federated_id}@example.com"),
            name: format!("Name {federated_id}"),
            login: format!("login-{federated_id}"),
            avatar_url: format!("https://example.com/{federated_id}.png"),
            federated_id: federated_id.to_string(),
            admin: false,
            verified: true,
            provider: String::from("github"),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_saved_user(users: Users) {
        // arrange
        let saved = users.save(user("1", 1)).await.unwrap();

        // act
        let found = users.find("1".into(), "github".into()).await.unwrap();

        // assert
        assert_eq!(saved.login, found.login);
        assert_eq!(saved.email, found.email);
        assert_eq!(saved.created, found.created);
    }

    #[rstest]
    #[tokio::test]
    async fn save_again_updates_profile_but_keeps_admin(users: Users) {
        // arrange
        users
            .save(User {
                admin: true,
                ..user("1", 1)
            })
            .await
            .unwrap();

        // act
        users
            .save(User {
                name: String::from("Renamed"),
                ..user("1", 1)
            })
            .await
            .unwrap();

        // assert
        let found = users.find("1".into(), "github".into()).await.unwrap();
        assert_eq!("Renamed", found.name);
        assert!(found.admin);
        assert_eq!(1, users.count().await.unwrap());
    }

    #[rstest]
    #[tokio::test]
    async fn all_lists_newest_first(users: Users) {
        // arrange
        users.save(user("old", 1)).await.unwrap();
        users.save(user("new", 2)).await.unwrap();

        // act
        let all = users.all().await.unwrap();

        // assert
        let ids: Vec<&str> = all.iter().map(|u| u.federated_id.as_str()).collect();
        assert_eq!(vec!["new", "old"], ids);
    }
}
