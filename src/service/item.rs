use std::sync::Arc;

use uuid::Uuid;

use crate::{
    config::Config,
    model::{
        error::{AppError, AppErrorCode},
        item::Item,
    },
    repository::Repository,
};

pub struct ItemService {
    repo: Arc<Repository>,
}

impl ItemService {
    pub fn new(_: Arc<Config>, repo: Arc<Repository>) -> Self {
        Self { repo }
    }

    pub async fn get(&self, id: String) -> Result<Item, AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }
        self.repo.item.get(id).await
    }

    pub async fn list(&self) -> Result<Vec<Item>, AppError> {
        self.repo.item.list().await
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
        self.repo.item.add(new_item).await
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

        self.repo.item.update(id, name).await
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }

        self.repo.item.delete(id).await
    }
}
