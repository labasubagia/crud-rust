use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Mutex;

use crate::model::{
    error::{AppError, AppErrorCode},
    item::Item,
};

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ItemRepository: Send + Sync {
    async fn add(&self, item: Item) -> Result<Item, AppError>;
    async fn list(&self) -> Result<Vec<Item>, AppError>;
    async fn get(&self, id: &str) -> Result<Item, AppError>;
    async fn update(&self, id: &str, name: String) -> Result<Item, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

pub struct InMemoryItemRepository {
    pub items: Mutex<Vec<Item>>,
}

impl Default for InMemoryItemRepository {
    fn default() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
        }
    }
}

impl InMemoryItemRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ItemRepository for InMemoryItemRepository {
    async fn add(&self, new_item: Item) -> Result<Item, AppError> {
        match self.items.lock() {
            Ok(mut items) => {
                let cur = items.iter().find(|item| item.name == new_item.name);
                match cur {
                    Some(item) => Ok(item.clone()),
                    None => {
                        items.push(new_item.clone());
                        Ok(new_item)
                    }
                }
            }
            Err(e) => Err(AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to lock items".to_string(),
            }),
        }
    }

    async fn list(&self) -> Result<Vec<Item>, AppError> {
        match self.items.lock() {
            Ok(items) => Ok(items.clone()),
            Err(e) => Err(AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to lock items".to_string(),
            }),
        }
    }

    async fn get(&self, id: &str) -> Result<Item, AppError> {
        match self.items.lock() {
            Ok(items) => {
                let item = items.iter().find(|item| item.id == id);
                match item {
                    Some(item) => Ok(item.clone()),
                    None => Err(AppError {
                        code: AppErrorCode::NotFound,
                        message: format!("Item with id {} not found", id),
                    }),
                }
            }
            Err(e) => Err(AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to lock items".to_string(),
            }),
        }
    }

    async fn update(&self, id: &str, name: String) -> Result<Item, AppError> {
        match self.items.lock() {
            Ok(mut items) => {
                let index = match items.iter().position(|item| item.id == id) {
                    Some(index) => index,
                    None => {
                        return Err(AppError {
                            code: AppErrorCode::NotFound,
                            message: format!("Item with id {} not found", id),
                        });
                    }
                };
                let cur = match items.get(index) {
                    Some(item) => item.clone(),
                    None => {
                        return Err(AppError {
                            code: AppErrorCode::NotFound,
                            message: format!("Item with id {} not found", id),
                        });
                    }
                };

                let updated_item = Item { name, ..cur };
                items[index] = updated_item.clone();
                Ok(updated_item)
            }
            Err(e) => Err(AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to lock items".to_string(),
            }),
        }
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        match self.items.lock() {
            Ok(mut items) => {
                *items = items
                    .clone()
                    .into_iter()
                    .filter(|item| item.id != id)
                    .collect();
                Ok(())
            }
            Err(e) => Err(AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to lock items".to_string(),
            }),
        }
    }
}

pub struct PostgresItemRepository {
    db: PgPool,
}

impl PostgresItemRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ItemRepository for PostgresItemRepository {
    async fn add(&self, item: Item) -> Result<Item, AppError> {
        let row = sqlx::query_as!(
            Item,
            r#"
                INSERT INTO items (id, name)
                VALUES ($1, $2)
                ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id, name
            "#,
            item.id,
            item.name
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError {
            code: AppErrorCode::InternalError(e.to_string()),
            message: "Failed to upsert item".to_string(),
        })?;
        Ok(row)
    }

    async fn list(&self) -> Result<Vec<Item>, AppError> {
        let rows = sqlx::query_as!(Item, r#"SELECT id, name FROM items ORDER BY name ASC"#)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to fetch items".to_string(),
            })?;
        Ok(rows)
    }

    async fn get(&self, id: &str) -> Result<Item, AppError> {
        let row = sqlx::query_as!(Item, r#"SELECT id, name FROM items WHERE id = $1"#, id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to fetch item".to_string(),
            })?;
        match row {
            Some(row) => Ok(row),
            None => Err(AppError {
                code: AppErrorCode::NotFound,
                message: format!("Item with id {} not found", id),
            }),
        }
    }

    async fn update(&self, id: &str, name: String) -> Result<Item, AppError> {
        let row = sqlx::query_as!(
            Item,
            r#"
                UPDATE items
                SET name = $2
                WHERE id = $1
                RETURNING id, name
            "#,
            id,
            name
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError {
            code: AppErrorCode::InternalError(e.to_string()),
            message: "Failed to update item".to_string(),
        })?;
        match row {
            Some(row) => Ok(row),
            None => Err(AppError {
                code: AppErrorCode::NotFound,
                message: format!("Item with id {} not found", id),
            }),
        }
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM items WHERE id = $1", id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to delete item".to_string(),
            })?;
        Ok(())
    }
}
