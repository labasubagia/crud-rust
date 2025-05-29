use uuid::Uuid;

use crate::model::{
    error::{AppError, AppErrorCode},
    item::Item,
};

use super::Service;

impl Service {
    pub async fn get_item(&self, id: String) -> Result<Item, AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }
        self.repo.get_item(id).await
    }

    pub async fn list_item(&self) -> Result<Vec<Item>, AppError> {
        self.repo.list_item().await
    }

    pub async fn create_item(&self, name: String) -> Result<Item, AppError> {
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
        self.repo.add_item(new_item).await
    }

    pub async fn update_item(&self, id: String, name: String) -> Result<Item, AppError> {
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

        self.repo.update_item(id, name).await
    }

    pub async fn delete_item(&self, id: String) -> Result<(), AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError {
                code: AppErrorCode::InvalidInput,
                message: "Item ID cannot be empty".to_string(),
            });
        }

        self.repo.delete_item(id).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{config::Config, repository::registry::MockRepository};

    use super::*;

    #[tokio::test]
    async fn test_create_item() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_add_item()
            .withf(|item: &Item| item.name == "test item")
            .returning(|item| Box::pin(async move { Ok(item) }));

        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));
        let item = service
            .create_item("Test Item".to_string())
            .await
            .expect("failed to create item");
        assert_eq!(item.name, "test item");
    }

    #[tokio::test]
    async fn test_get_item() {
        let mut mock_repo = MockRepository::new();

        let item = Item {
            id: "123".to_string(),
            name: "test item".to_string(),
        };
        mock_repo
            .expect_get_item()
            .withf(|id| id == "123")
            .returning(move |_| {
                Box::pin({
                    let value = item.clone();
                    async move { Ok(value) }
                })
            });

        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));
        let fetched_item = service
            .get_item("123".to_string())
            .await
            .expect("failed to get item");
        assert_eq!(fetched_item.id, "123");
        assert_eq!(fetched_item.name, "test item");
    }

    #[tokio::test]
    async fn test_list_items() {
        let mut mock_repo = MockRepository::new();

        let items = vec![
            Item {
                id: "1".to_string(),
                name: "item one".to_string(),
            },
            Item {
                id: "2".to_string(),
                name: "item two".to_string(),
            },
        ];
        mock_repo.expect_list_item().returning(move || {
            Box::pin({
                let value = items.clone();
                async move { Ok(value.clone()) }
            })
        });
        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));

        let fetched_items = service.list_item().await.expect("failed to list items");
        assert_eq!(fetched_items.len(), 2);
        assert_eq!(fetched_items[0].name, "item one");
        assert_eq!(fetched_items[1].name, "item two");
    }

    #[tokio::test]
    async fn test_update_item() {
        let mut mock_repo = MockRepository::new();

        let item = Item {
            id: "123".to_string(),
            name: "updated item".to_string(),
        };
        mock_repo
            .expect_update_item()
            .withf(|id, name| id == "123" && name == "updated item")
            .returning(move |_, _| {
                Box::pin({
                    let value = item.clone();
                    async move { Ok(value.clone()) }
                })
            });

        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));

        let updated_item = service
            .update_item("123".to_string(), "Updated Item".to_string())
            .await
            .expect("failed to update item");
        assert_eq!(updated_item.id, "123");
        assert_eq!(updated_item.name, "updated item");
    }

    #[tokio::test]
    async fn test_delete_item() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_delete_item()
            .withf(|id| id == "123")
            .returning(move |_| Box::pin(async move { Ok(()) }));

        let service = Service::new(Arc::new(Config::new()), Arc::new(mock_repo));
        let result = service.delete_item("123".to_string()).await;
        assert!(result.is_ok());
    }
}
