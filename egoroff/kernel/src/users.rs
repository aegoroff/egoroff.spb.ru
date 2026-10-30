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
