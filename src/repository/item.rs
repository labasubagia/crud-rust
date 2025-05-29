use sqlx::PgPool;

use crate::model::{
    error::{AppError, AppErrorCode},
    item::Item,
};

pub async fn add_item(db: &PgPool, item: Item) -> Result<Item, AppError> {
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
    .fetch_one(db)
    .await
    .map_err(|e| AppError {
        code: AppErrorCode::InternalError(e.to_string()),
        message: "Failed to upsert item".to_string(),
    })?;
    Ok(row)
}

pub async fn list_item(db: &PgPool) -> Result<Vec<Item>, AppError> {
    let rows = sqlx::query_as!(Item, r#"SELECT id, name FROM items ORDER BY name ASC"#)
        .fetch_all(db)
        .await
        .map_err(|e| AppError {
            code: AppErrorCode::InternalError(e.to_string()),
            message: "Failed to fetch items".to_string(),
        })?;
    Ok(rows)
}

pub async fn get_item(db: &PgPool, id: &str) -> Result<Item, AppError> {
    let row = sqlx::query_as!(Item, r#"SELECT id, name FROM items WHERE id = $1"#, id)
        .fetch_optional(db)
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

pub async fn update_item(db: &PgPool, id: &str, name: String) -> Result<Item, AppError> {
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
    .fetch_optional(db)
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

pub async fn delete_item(db: &PgPool, id: &str) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM items WHERE id = $1", id)
        .execute(db)
        .await
        .map_err(|e| AppError {
            code: AppErrorCode::InternalError(e.to_string()),
            message: "Failed to delete item".to_string(),
        })?;
    Ok(())
}
