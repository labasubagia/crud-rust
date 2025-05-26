use async_trait::async_trait;
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

pub struct PostgresItemRepository<'e, E>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres> + Send + Sync + Copy,
{
    pub executor: E,
    _marker: std::marker::PhantomData<&'e ()>,
}

impl<'e, E> PostgresItemRepository<'e, E>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres> + Send + Sync + Copy,
{
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<'e, E> ItemRepository for PostgresItemRepository<'e, E>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres> + Send + Sync + Copy,
{
    async fn add(&self, item: Item) -> Result<Item, AppError> {
        use sqlx::Row;
        let query = r#"
            INSERT INTO items (id, name)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name
            RETURNING id, name
        "#;
        let row = sqlx::query(query)
            .bind(&item.id)
            .bind(&item.name)
            .fetch_one(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to upsert item".to_string(),
            })?;
        Ok(Item {
            id: row.try_get("id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            ..item
        })
    }

    async fn list(&self) -> Result<Vec<Item>, AppError> {
        use sqlx::Row;
        let query = r#"
            SELECT id, name FROM items
        "#;
        let rows = sqlx::query(query)
            .fetch_all(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to fetch items".to_string(),
            })?;
        let items = rows
            .into_iter()
            .map(|row| Item {
                id: row.try_get("id").unwrap_or_default(),
                name: row.try_get("name").unwrap_or_default(),
            })
            .collect();
        Ok(items)
    }

    async fn get(&self, id: &str) -> Result<Item, AppError> {
        use sqlx::Row;
        let query = r#"
            SELECT id, name FROM items WHERE id = $1
        "#;
        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to fetch item".to_string(),
            })?;
        match row {
            Some(row) => Ok(Item {
                id: row.try_get("id").unwrap_or_default(),
                name: row.try_get("name").unwrap_or_default(),
            }),
            None => Err(AppError {
                code: AppErrorCode::NotFound,
                message: format!("Item with id {} not found", id),
            }),
        }
    }

    async fn update(&self, id: &str, name: String) -> Result<Item, AppError> {
        use sqlx::Row;
        let query = r#"
            UPDATE items
            SET name = $2
            WHERE id = $1
            RETURNING id, name
        "#;
        let row = sqlx::query(query)
            .bind(id)
            .bind(&name)
            .fetch_optional(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to update item".to_string(),
            })?;
        match row {
            Some(row) => Ok(Item {
                id: row.try_get("id").unwrap_or_default(),
                name: row.try_get("name").unwrap_or_default(),
            }),
            None => Err(AppError {
                code: AppErrorCode::NotFound,
                message: format!("Item with id {} not found", id),
            }),
        }
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let query = r#"
            DELETE FROM items WHERE id = $1
        "#;
        sqlx::query(query)
            .bind(id)
            .execute(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to delete item".to_string(),
            })?;
        Ok(())
    }
}
