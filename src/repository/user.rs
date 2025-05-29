use sqlx::PgPool;

use crate::model::{
    error::{AppError, AppErrorCode},
    user::User,
};

pub async fn add_user(db: &PgPool, user: User) -> Result<User, AppError> {
    let row = sqlx::query_as!(
        User,
        r#"
            INSERT INTO users (id, email)
            VALUES ($1, $2)
            ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email
            RETURNING id, email
        "#,
        user.id,
        user.email,
    )
    .fetch_one(db)
    .await
    .map_err(|e| AppError {
        code: AppErrorCode::InternalError(e.to_string()),
        message: "Failed to upsert user".to_string(),
    })?;
    Ok(row)
}

pub async fn list_user(db: &PgPool) -> Result<Vec<User>, AppError> {
    let rows = sqlx::query_as!(User, r#"SELECT id, email FROM users ORDER BY email ASC"#)
        .fetch_all(db)
        .await
        .map_err(|e| AppError {
            code: AppErrorCode::InternalError(e.to_string()),
            message: "Failed to fetch users".to_string(),
        })?;
    Ok(rows)
}

pub async fn get_user(db: &PgPool, id: &str) -> Result<User, AppError> {
    let row = sqlx::query_as!(User, r#"SELECT id, email FROM users WHERE id = $1"#, id)
        .fetch_optional(db)
        .await
        .map_err(|e| AppError {
            code: AppErrorCode::InternalError(e.to_string()),
            message: "Failed to fetch user".to_string(),
        })?;
    match row {
        Some(row) => Ok(row),
        None => Err(AppError {
            code: AppErrorCode::NotFound,
            message: format!("User with id {} not found", id),
        }),
    }
}

pub async fn update_user(db: &PgPool, id: &str, email: String) -> Result<User, AppError> {
    let row = sqlx::query_as!(
        User,
        r#"
            UPDATE users
            SET email = $2
            WHERE id = $1
            RETURNING id, email
        "#,
        id,
        email
    )
    .fetch_optional(db)
    .await
    .map_err(|e| AppError {
        code: AppErrorCode::InternalError(e.to_string()),
        message: "Failed to update user".to_string(),
    })?;
    match row {
        Some(row) => Ok(row),
        None => Err(AppError {
            code: AppErrorCode::NotFound,
            message: format!("User with id {} not found", id),
        }),
    }
}

pub async fn delete_user(db: &PgPool, id: &str) -> Result<(), AppError> {
    sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id)
        .execute(db)
        .await
        .map_err(|e| AppError {
            code: AppErrorCode::InternalError(e.to_string()),
            message: "Failed to delete user".to_string(),
        })?;
    Ok(())
}
