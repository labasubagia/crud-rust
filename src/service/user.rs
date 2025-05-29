use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::{
    error::{AppError, AppErrorCode},
    user::User,
};

use super::Service;

#[derive(Deserialize, Serialize, Clone)]
pub struct CreateUser {
    pub email: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UpdateUser {
    pub email: String,
}

impl Service {
    pub async fn add_user(&self, payload: CreateUser) -> Result<User, AppError> {
        let email = payload.email.trim().to_string();
        if email.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Email is required".into(),
            });
        }

        let user = User {
            id: Uuid::new_v4().to_string(),
            email,
        };
        self.repo.add_user(user).await
    }

    pub async fn list_user(&self) -> Result<Vec<User>, AppError> {
        self.repo.list_user().await
    }

    pub async fn get_user(&self, id: &str) -> Result<User, AppError> {
        if id.is_empty() || Uuid::parse_str(id).is_err() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Invalid user ID format".into(),
            });
        }
        self.repo.get_user(id).await
    }

    pub async fn update_user(&self, id: &str, payload: UpdateUser) -> Result<User, AppError> {
        if id.is_empty() || Uuid::parse_str(id).is_err() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Invalid user ID format".into(),
            });
        }
        let email = payload.email.trim().to_string();
        if email.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Email cannot be empty".into(),
            });
        }
        self.repo.update_user(id, email).await
    }

    pub async fn delete_user(&self, id: &str) -> Result<(), AppError> {
        if id.is_empty() || Uuid::parse_str(id).is_err() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Invalid user ID format".into(),
            });
        }

        self.repo.delete_user(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::model::user::User;
    use crate::repository::registry::MockRepository;
    use crate::service::user::UpdateUser;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_add_user() {
        let mut mock_repo = MockRepository::new();
        let payload = CreateUser {
            email: "test@example.com".to_string(),
        };
        mock_repo
            .expect_add_user()
            .withf(|u| u.email == "test@example.com")
            .returning(|u| Box::pin(async move { Ok(u) }));

        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));
        let result = service.add_user(payload).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_list_users() {
        let mut mock_repo = MockRepository::new();
        let users = vec![User {
            id: "1".to_string(),
            email: "a@b.com".to_string(),
        }];
        let users_clone = users.clone();
        mock_repo.expect_list_user().returning(move || {
            let users = users_clone.clone();
            Box::pin(async move { Ok(users) })
        });

        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));
        let result = service.list_user().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut mock_repo = MockRepository::new();
        let user = User {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            email: "a@b.com".to_string(),
        };
        let user_clone = user.clone();
        mock_repo
            .expect_get_user()
            .withf(|id| id == "123e4567-e89b-12d3-a456-426614174000")
            .returning(move |_| {
                let user = user_clone.clone();
                Box::pin(async move { Ok(user) })
            });

        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));

        let result = service.get_user(&user.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "a@b.com");
    }

    #[tokio::test]
    async fn test_update_user() {
        let mut mock_repo = MockRepository::new();
        let update_user = UpdateUser {
            email: "new@b.com".to_string(),
        };
        let user = User {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            email: update_user.email.clone(),
        };
        let user_clone = user.clone();
        mock_repo
            .expect_update_user()
            .withf(|id, email| id == "123e4567-e89b-12d3-a456-426614174000" && email == "new@b.com")
            .returning(move |_, _| {
                let user = user_clone.clone();
                Box::pin(async move { Ok(user) })
            });

        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));
        let result = service.update_user(&user.id, update_user).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "new@b.com");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let mut mock_repo = MockRepository::new();
        mock_repo
            .expect_delete_user()
            .withf(|id| id == "123e4567-e89b-12d3-a456-426614174000")
            .returning(|_| Box::pin(async move { Ok(()) }));

        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));
        let result = service
            .delete_user("123e4567-e89b-12d3-a456-426614174000")
            .await;
        assert!(result.is_ok());
    }
}
