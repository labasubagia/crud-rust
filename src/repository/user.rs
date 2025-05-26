use async_trait::async_trait;

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

pub struct PostgresUserRepository<'e, E>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres> + Send + Sync + Copy,
{
    pub executor: E,
    _marker: std::marker::PhantomData<&'e ()>,
}

impl<'e, E> PostgresUserRepository<'e, E>
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
impl<'e, E> UserRepository for PostgresUserRepository<'e, E>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres> + Send + Sync + Copy,
{
    async fn add(&self, user: User) -> Result<User, AppError> {
        use sqlx::Row;
        let query = r#"
            INSERT INTO users (id, email)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET email = EXCLUDED.email
            RETURNING id, email
        "#;
        let row = sqlx::query(query)
            .bind(&user.id)
            .bind(&user.email)
            .fetch_one(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to upsert user".to_string(),
            })?;
        Ok(User {
            id: row.try_get("id").unwrap_or_default(),
            email: row.try_get("email").unwrap_or_default(),
            ..user
        })
    }

    async fn list(&self) -> Result<Vec<User>, AppError> {
        use sqlx::Row;
        let query = r#"
            SELECT id, email FROM users
        "#;
        let rows = sqlx::query(query)
            .fetch_all(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to fetch users".to_string(),
            })?;
        let users = rows
            .into_iter()
            .map(|row| User {
                id: row.try_get("id").unwrap_or_default(),
                email: row.try_get("email").unwrap_or_default(),
            })
            .collect();
        Ok(users)
    }

    async fn get(&self, id: &str) -> Result<User, AppError> {
        use sqlx::Row;
        let query = r#"
            SELECT id, email FROM users WHERE id = $1
        "#;
        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to fetch user".to_string(),
            })?;
        match row {
            Some(row) => Ok(User {
                id: row.try_get("id").unwrap_or_default(),
                email: row.try_get("email").unwrap_or_default(),
            }),
            None => Err(AppError {
                code: AppErrorCode::NotFound,
                message: format!("User with id {} not found", id),
            }),
        }
    }

    async fn update(&self, id: &str, email: String) -> Result<User, AppError> {
        use sqlx::Row;
        let query = r#"
            UPDATE users
            SET email = $2
            WHERE id = $1
            RETURNING id, email
        "#;
        let row = sqlx::query(query)
            .bind(id)
            .bind(&email)
            .fetch_optional(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to update user".to_string(),
            })?;
        match row {
            Some(row) => Ok(User {
                id: row.try_get("id").unwrap_or_default(),
                email: row.try_get("email").unwrap_or_default(),
            }),
            None => Err(AppError {
                code: AppErrorCode::NotFound,
                message: format!("User with id {} not found", id),
            }),
        }
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let query = r#"
            DELETE FROM users WHERE id = $1
        "#;
        sqlx::query(query)
            .bind(id)
            .execute(self.executor)
            .await
            .map_err(|e| AppError {
                code: AppErrorCode::InternalError(e.to_string()),
                message: "Failed to delete user".to_string(),
            })?;
        Ok(())
    }
}
