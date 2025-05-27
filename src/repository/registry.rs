use std::sync::Arc;

use sqlx::PgPool;

use super::{
    item::{ItemRepository, PostgresItemRepository},
    user::{PostgresUserRepository, UserRepository},
};

pub trait Repository: Send + Sync {
    fn item(&self) -> Arc<dyn ItemRepository>;
    fn user(&self) -> Arc<dyn UserRepository>;
}

pub struct PostgresRepository {
    pub item: Arc<PostgresItemRepository>,
    pub user: Arc<PostgresUserRepository>,
}

#[cfg_attr(test, mockall::automock)]
impl Repository for PostgresRepository {
    fn item(&self) -> Arc<dyn ItemRepository> {
        self.item.clone()
    }

    fn user(&self) -> Arc<dyn UserRepository> {
        self.user.clone()
    }
}

impl PostgresRepository {
    pub fn new(db: PgPool) -> Self {
        Self {
            item: Arc::new(PostgresItemRepository::new(db.clone())),
            user: Arc::new(PostgresUserRepository::new(db.clone())),
        }
    }
}
