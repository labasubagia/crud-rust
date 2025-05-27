use async_trait::async_trait;
use sqlx::PgPool;

use crate::model::{
    error::{AppError, AppErrorCode},
    user::User,
};

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait UserRepository: Send + Sync {
    async fn add(&self, user: User) -> Result<User, AppError>;
    async fn list(&self) -> Result<Vec<User>, AppError>;
    async fn get(&self, id: &str) -> Result<User, AppError>;
    async fn update(&self, id: &str, name: String) -> Result<User, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

pub struct PostgresUserRepository {
    db: PgPool,
}

impl PostgresUserRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn add(&self, user: User) -> Result<User, AppError> {
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
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError {
            code: AppErrorCode::InternalError(e.to_string()),
            message: "Failed to upsert user".to_string(),
        })?;
        Ok(row)
    }

    async fn list(&self) -> Result<Vec<User>, AppError> {
        let rows = sqlx::query_as!(User, r#"SELECT id, email FROM users ORDER BY email ASC"#)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to fetch users".to_string(),
            })?;
        Ok(rows)
    }

    async fn get(&self, id: &str) -> Result<User, AppError> {
        let row = sqlx::query_as!(User, r#"SELECT id, email FROM users WHERE id = $1"#, id)
            .fetch_optional(&self.db)
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

    async fn update(&self, id: &str, email: String) -> Result<User, AppError> {
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
        .fetch_optional(&self.db)
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

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to delete user".to_string(),
            })?;
        Ok(())
    }
}
