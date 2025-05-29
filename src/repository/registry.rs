use sqlx::PgPool;

use crate::model::{error::AppError, item::Item, user::User};

use super::item::{add_item, delete_item, get_item, list_item, update_item};
use super::user::{add_user, delete_user, get_user, list_user, update_user};

#[async_trait::async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait Repository: Sync + Send {
    async fn add_item(&self, item: Item) -> Result<Item, AppError>;
    async fn list_item(&self) -> Result<Vec<Item>, AppError>;
    async fn get_item(&self, id: &str) -> Result<Item, AppError>;
    async fn update_item(&self, id: &str, name: String) -> Result<Item, AppError>;
    async fn delete_item(&self, id: &str) -> Result<(), AppError>;

    async fn add_user(&self, user: User) -> Result<User, AppError>;
    async fn list_user(&self) -> Result<Vec<User>, AppError>;
    async fn get_user(&self, id: &str) -> Result<User, AppError>;
    async fn update_user(&self, id: &str, name: String) -> Result<User, AppError>;
    async fn delete_user(&self, id: &str) -> Result<(), AppError>;
}

pub struct PostgresRepository {
    pub db: PgPool,
}

impl PostgresRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Repository for PostgresRepository {
    async fn add_item(&self, item: Item) -> Result<Item, AppError> {
        add_item(&self.db, item).await
    }

    async fn list_item(&self) -> Result<Vec<Item>, AppError> {
        list_item(&self.db).await
    }

    async fn get_item(&self, id: &str) -> Result<Item, AppError> {
        get_item(&self.db, id).await
    }

    async fn update_item(&self, id: &str, name: String) -> Result<Item, AppError> {
        update_item(&self.db, id, name).await
    }

    async fn delete_item(&self, id: &str) -> Result<(), AppError> {
        delete_item(&self.db, id).await
    }

    async fn add_user(&self, user: User) -> Result<User, AppError> {
        add_user(&self.db, user).await
    }

    async fn list_user(&self) -> Result<Vec<User>, AppError> {
        list_user(&self.db).await
    }

    async fn get_user(&self, id: &str) -> Result<User, AppError> {
        get_user(&self.db, id).await
    }

    async fn update_user(&self, id: &str, name: String) -> Result<User, AppError> {
        update_user(&self.db, id, name).await
    }

    async fn delete_user(&self, id: &str) -> Result<(), AppError> {
        delete_user(&self.db, id).await
    }
}
