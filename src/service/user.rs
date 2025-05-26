use std::sync::Arc;

use uuid::Uuid;

use crate::{
    config::Config,
    model::{error::AppError, user::User},
    repository::Repository,
};

pub struct UserService {
    repo: Arc<Repository>,
}

impl UserService {
    pub fn new(_: Arc<Config>, repo: Arc<Repository>) -> Self {
        Self { repo }
    }

    pub async fn test_trx(&self) -> Result<(), AppError> {
        let id = Uuid::new_v4().to_string();

        self.repo
            .user
            .add(User {
                id: id.clone(),
                email: Uuid::new_v4().to_string(),
            })
            .await
            .map_err(|e| AppError {
                code: e.code,
                message: format!("Failed to add user: {}", e.message),
            })?;

        // Simulate a failure to test transaction rollback
        if "".is_empty() {
            return Err(AppError {
                code: crate::model::error::AppErrorCode::InvalidInput,
                message: "User cannot be empty".to_string(),
            });
        }

        self.repo
            .item
            .add(crate::model::item::Item {
                id: id.clone(),
                name: Uuid::new_v4().to_string(),
            })
            .await
            .map_err(|e| AppError {
                code: e.code,
                message: format!("Failed to add item: {}", e.message),
            })?;

        Ok(())
    }
}
