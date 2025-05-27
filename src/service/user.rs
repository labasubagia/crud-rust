use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::Config,
    model::{
        error::{AppError, AppErrorCode},
        user::User,
    },
    repository::Repository,
};

#[derive(Deserialize, Serialize, Clone)]
pub struct CreateUser {
    pub email: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UpdateUser {
    pub email: String,
}

pub struct UserService {
    repo: Arc<dyn Repository>,
}

impl UserService {
    pub fn new(_: Arc<Config>, repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }

    pub async fn add(&self, payload: CreateUser) -> Result<User, AppError> {
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
        self.repo.user().add(user).await
    }

    pub async fn list(&self) -> Result<Vec<User>, AppError> {
        self.repo.user().list().await
    }

    pub async fn get(&self, id: &str) -> Result<User, AppError> {
        if id.is_empty() || Uuid::parse_str(id).is_err() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Invalid user ID format".into(),
            });
        }
        self.repo.user().get(id).await
    }

    pub async fn update(&self, id: &str, payload: UpdateUser) -> Result<User, AppError> {
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
        self.repo.user().update(id, email).await
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        if id.is_empty() || Uuid::parse_str(id).is_err() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Invalid user ID format".into(),
            });
        }

        self.repo.user().delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::model::user::User;
    use crate::repository::registry::MockPostgresRepository;
    use crate::repository::{item::MockItemRepository, user::MockUserRepository};
    use crate::service::user::UpdateUser;
    use std::sync::Arc;

    fn make_service(mock_user_repo: Arc<MockUserRepository>) -> UserService {
        let mock_item_repo = Arc::new(MockItemRepository::new());
        let mut mock_repo = MockPostgresRepository::new();
        mock_repo
            .expect_user()
            .returning(move || mock_user_repo.clone());
        mock_repo
            .expect_item()
            .returning(move || mock_item_repo.clone());
        UserService::new(Arc::new(Config::default()), Arc::new(mock_repo))
    }

    #[tokio::test]
    async fn test_add_user() {
        let mut mock_user_repo = MockUserRepository::new();
        let payload = CreateUser {
            email: "test@example.com".to_string(),
        };
        mock_user_repo
            .expect_add()
            .withf(|u| u.email == "test@example.com")
            .returning(|u| Box::pin(async move { Ok(u) }));
        let service = make_service(Arc::new(mock_user_repo));
        let result = service.add(payload).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_list_users() {
        let mut mock_user_repo = MockUserRepository::new();
        let users = vec![User {
            id: "1".to_string(),
            email: "a@b.com".to_string(),
        }];
        let users_clone = users.clone();
        mock_user_repo.expect_list().returning(move || {
            let users = users_clone.clone();
            Box::pin(async move { Ok(users) })
        });
        let service = make_service(Arc::new(mock_user_repo));
        let result = service.list().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut mock_user_repo = MockUserRepository::new();
        let user = User {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            email: "a@b.com".to_string(),
        };
        let user_clone = user.clone();
        mock_user_repo
            .expect_get()
            .withf(|id| id == "123e4567-e89b-12d3-a456-426614174000")
            .returning(move |_| {
                let user = user_clone.clone();
                Box::pin(async move { Ok(user) })
            });
        let service = make_service(Arc::new(mock_user_repo));
        let result = service.get(&user.id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "a@b.com");
    }

    #[tokio::test]
    async fn test_update_user() {
        let mut mock_user_repo = MockUserRepository::new();
        let update_user = UpdateUser {
            email: "new@b.com".to_string(),
        };
        let user = User {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            email: update_user.email.clone(),
        };
        let user_clone = user.clone();
        mock_user_repo
            .expect_update()
            .withf(|id, email| id == "123e4567-e89b-12d3-a456-426614174000" && email == "new@b.com")
            .returning(move |_, _| {
                let user = user_clone.clone();
                Box::pin(async move { Ok(user) })
            });
        let service = make_service(Arc::new(mock_user_repo));
        let result = service.update(&user.id, update_user).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "new@b.com");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let mut mock_user_repo = MockUserRepository::new();
        mock_user_repo
            .expect_delete()
            .withf(|id| id == "123e4567-e89b-12d3-a456-426614174000")
            .returning(|_| Box::pin(async move { Ok(()) }));
        let service = make_service(Arc::new(mock_user_repo));
        let result = service.delete("123e4567-e89b-12d3-a456-426614174000").await;
        assert!(result.is_ok());
    }
}
