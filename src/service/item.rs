use std::sync::Arc;

use uuid::Uuid;

use crate::{
    config::Config,
    model::{
        error::{AppError, AppErrorCode},
        item::Item,
    },
    repository::item::ItemRepository,
};

pub struct ItemService<R: ItemRepository> {
    pub config: Arc<Config>,
    repo: Arc<R>,
}

impl<R: ItemRepository> ItemService<R> {
    pub fn new(config: Arc<Config>, repo: Arc<R>) -> Self {
        ItemService { config, repo }
    }

    pub async fn get(&self, id: String) -> Result<Item, AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }
        self.repo.get(id).await
    }

    pub async fn list(&self) -> Result<Vec<Item>, AppError> {
        self.repo.list().await
    }

    pub async fn create(&self, name: String) -> Result<Item, AppError> {
        let name = name.trim().to_lowercase();
        if name.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item name cannot be empty".to_string(),
            });
        }

        let new_item = Item {
            id: Uuid::new_v4().to_string(),
            name,
        };
        self.repo.add(new_item).await
    }

    pub async fn update(&self, id: String, name: String) -> Result<Item, AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }

        let name = name.trim().to_lowercase();
        if name.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item name cannot be empty".to_string(),
            });
        }

        self.repo.update(id, name).await
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }

        self.repo.delete(id).await
    }
}
